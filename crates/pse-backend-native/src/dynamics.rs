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

mod anchored;
pub use anchored::Anchored;
#[cfg(feature = "idas")]
mod idas;
#[cfg(feature = "diffsol")]
mod integrator;
#[cfg(feature = "diffsol")]
mod linear;

/// Registry vocabularies of the dynamics settings (ADR-0110, ADR-0115 Outcome 3): the
/// requested integration algorithm (`auto` resolves from trial requirements), the contract
/// for domain errors at internal trial points, Diffsol's library-owned time-stepping scheme
/// and the sparse factorization of its Newton matrices, IDAS's forward-sensitivity
/// corrector (`IDASensInit`), its consistent initialization (`IDACalcIC`), the sign a
/// normalized state keeps (`IDASetConstraints`) and the guard crossings that trigger an
/// event (`IDASetRootDirection`).
pub use pse_model::generated::enums::{
    DiffsolLinear, DiffsolMethod, DynamicSensitivity, DynamicsMethod as Method, EventDirection,
    IdasInitialization, SensitivityCorrector, StateSign, TrialPolicy,
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
/// Forward checkpoints of the adjoint backward pass (ADR-0110 item 3), a versioned boundary
/// document (ADR-0116 Outcome 6): the version is required, and absent fields take these
/// defaults. The backward pass replays at most `steps_between_checkpoints` native steps
/// from a stored checkpoint, and at most `max_checkpoints` are held at once, at least two
/// per scheduled segment; a forward pass that needs more stops with a typed memory limit.
/// Their estimated bytes are charged against the caller's memory allowance before any
/// native work.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AdjointSettings {
    /// Document version.
    pub version: Version<1>,
    /// Native forward steps between stored checkpoints (IDAS `IDAAdjInit`'s `Nd`).
    #[serde(default = "AdjointSettings::default_steps")]
    pub steps_between_checkpoints: PositiveCount,
    /// Largest number of forward checkpoints held at once.
    #[serde(default = "AdjointSettings::default_checkpoints")]
    pub max_checkpoints: PositiveCount,
}
impl AdjointSettings {
    const fn default_steps() -> PositiveCount {
        pse_model::scalar!(PositiveCount(250))
    }
    const fn default_checkpoints() -> PositiveCount {
        pse_model::scalar!(PositiveCount(400))
    }
}
impl Default for AdjointSettings {
    /// A checkpoint every 250 steps, at most 400 of them.
    ///
    /// ```
    /// use pse_backend_native::dynamics::AdjointSettings;
    ///
    /// let settings: AdjointSettings = serde_json::from_str(r#"{"version":1}"#).unwrap();
    /// assert_eq!(settings, AdjointSettings::default());
    /// assert_eq!(settings.steps_between_checkpoints.into_inner(), 250);
    /// let zero = r#"{"version":1,"max_checkpoints":0}"#;
    /// assert!(serde_json::from_str::<AdjointSettings>(zero).is_err());
    /// ```
    fn default() -> Self {
        Self {
            version: Version,
            steps_between_checkpoints: Self::default_steps(),
            max_checkpoints: Self::default_checkpoints(),
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
/// The native IDAS/KINSOL constraint code of a state sign.
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
/// Version 2 removes the per-state sign constraints: they derive from the authored bounds
/// ([`Contract::signs`], ADR-0119 Outcome 4).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IdasSettings {
    /// Document version.
    pub version: Version<2>,
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
    /// Public IDACalcIC phase controls, also applied to scheduled consistent restarts.
    #[serde(default)]
    pub initial_conditions: IdasInitialConditions,
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
    /// KLU over the analytic Jacobian, the simultaneous corrector and an initialization
    /// that keeps the requested differential states.
    fn default() -> Self {
        Self {
            version: Version,
            linear: IdasLinear::Klu,
            sensitivity: Self::default_sensitivity(),
            initialization: Self::default_initialization(),
            initial_conditions: IdasInitialConditions::default(),
        }
    }
}
/// Exact IDAS consistent-initialization controls; no physical time stepping is requested.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct IdasInitialConditions {
    /// Maximum attempted artificial step scales; applies only to AlgebraicAndRates.
    pub step_trials: Option<u32>,
    /// Maximum Jacobian setup attempts at each artificial scale.
    pub jacobian_attempts: u32,
    /// Maximum Newton iterations at each setup attempt.
    pub newton_iterations: u32,
    /// Positive IDASetNonlinConvCoefIC coefficient.
    pub convergence_coefficient: f64,
    /// Enable IDAS's IC line search.
    pub line_search: bool,
    /// Explicit maximum backtracks per Newton step, when line search is enabled.
    pub backtracks: Option<u32>,
    /// Positive scaled Newton-step floor for constraints and the enabled line search.
    pub step_tolerance: Option<f64>,
}
impl Default for IdasInitialConditions {
    fn default() -> Self {
        Self {
            step_trials: None,
            jacobian_attempts: 4,
            newton_iterations: 10,
            convergence_coefficient: 0.0033,
            line_search: true,
            backtracks: None,
            step_tolerance: None,
        }
    }
}
impl IdasInitialConditions {
    pub(crate) fn validate_for(
        &self,
        mode: IdasInitialization,
        sign_constraints: bool,
    ) -> Result<(), ProblemError> {
        if [
            Some(self.jacobian_attempts),
            Some(self.newton_iterations),
            self.step_trials,
            self.backtracks,
        ]
        .into_iter()
        .flatten()
        .any(|n| n == 0 || i32::try_from(n).is_err())
            || !positive(self.convergence_coefficient)
            || self.step_tolerance.is_some_and(|x| !positive(x))
        {
            return Err(contract("invalid IDAS consistent-initialization controls"));
        }
        if mode == IdasInitialization::SteadyStates && self.step_trials.is_some() {
            return Err(contract(
                "IC step-scale trials do not act in the all-state initialization mode",
            ));
        }
        if !self.line_search && self.backtracks.is_some() {
            return Err(contract(
                "IC line-search controls require the IC line search",
            ));
        }
        if !self.line_search && !sign_constraints && self.step_tolerance.is_some() {
            return Err(contract(
                "IC step tolerance requires the IC line search or authored sign constraints",
            ));
        }
        let scales = if mode == IdasInitialization::AlgebraicAndRates {
            self.step_trials.unwrap_or(5)
        } else {
            1
        };
        let iterations = u64::from(scales)
            .checked_mul(u64::from(self.jacobian_attempts))
            .and_then(|n| n.checked_mul(u64::from(self.newton_iterations)))
            .and_then(|n| n.checked_mul(2));
        if iterations.is_none_or(|n| n > std::ffi::c_long::MAX as u64) {
            return Err(contract(
                "composed IC iteration extent exceeds the native counter",
            ));
        }
        Ok(())
    }
}
/// One original semi-explicit consistent-state calculation, with no integration horizon.
#[derive(Clone, Debug)]
pub struct ConsistentInitialization {
    /// Original physical time at which the original residual is assessed.
    pub time: f64,
    /// Finite distinct time used only to orient/scale IDACalcIC, never integrated toward.
    pub toward: f64,
    /// Positive native weighted convergence tolerances.
    pub rtol: f64,
    /// Absolute native state tolerances in the contract's order.
    pub atol: Vec<f64>,
    /// Original residual tolerances in the same normalized residual order.
    pub residual_tolerances: Vec<f64>,
    /// Explicit role-preserving or all-state mode.
    pub mode: IdasInitialization,
    /// Analytic native Newton linear solver.
    pub linear: IdasLinear,
    /// Actual IC-only controls.
    pub controls: IdasInitialConditions,
    /// Whether original-domain trial failures are recoverable during the IC line search.
    pub trial_failures: TrialPolicy,
}
/// Finite native observation; its existence alone never qualifies a consistent state.
#[derive(Clone, Debug)]
pub struct ConsistentState {
    /// Original state coordinates in the authored order.
    pub state: Vec<f64>,
    /// Actual native rates in the same coordinate order.
    pub rates: Vec<f64>,
}
/// Independent assessment of the authored residual and requested role preservation.
#[derive(Clone, Debug)]
pub struct ConsistentStateAssessment {
    /// Actual original residual, differential rates minus RHS or minus algebraic RHS.
    pub residual: Vec<f64>,
    /// Every residual meets its explicitly requested tolerance.
    pub residual_satisfied: bool,
    /// AlgebraicAndRates kept every requested differential state; all-state mode is explicit.
    pub roles_preserved: bool,
    /// Every observed state meets the authored sign constraint.
    pub signs_satisfied: bool,
}
/// A native IC attempt and its original-space assessment, including failed native attempts.
#[derive(Debug)]
pub struct ConsistentStateReport {
    /// Original authored dynamics identity for this observation.
    pub identity: ContentHash,
    /// Original time at which the original residual was assessed.
    pub time: f64,
    /// Explicit mode actually supplied to IDACalcIC.
    pub mode: IdasInitialization,
    /// Whether the first actual original residual in this IC phase succeeded; unknown if uncalled.
    pub initial_residual: Option<bool>,
    /// Actual IDACalcIC return with terminal callback category applied separately.
    pub termination: crate::solve::NativeTermination,
    /// Authored initial values actually submitted.
    pub requested: Vec<f64>,
    /// Finite native observation after IDACalcIC, when retrievable.
    pub candidate: Option<ConsistentState>,
    /// Fresh original residual/role evidence; absent after a terminal latch.
    pub assessment: Option<ConsistentStateAssessment>,
    /// Actual callback/native work; unknown counters remain absent.
    pub evidence: crate::solve::Evidence,
    /// Actual IDAS backtrack operations, including failed IC attempts.
    pub backtracks: Option<u64>,
    /// Original typed native or callback cause, never replaced by a message wrapper.
    pub error: Option<ProblemError>,
    /// Independent assessment failure, kept separate from the native attempt cause.
    pub validation_error: Option<ProblemError>,
}
impl ConsistentStateReport {
    /// Known escaping numeric buffers; the caller separately reserves typed diagnostics.
    pub fn numeric_bytes(&self) -> usize {
        size_of::<Self>()
            + size_of::<f64>()
                * (self.requested.capacity()
                    + self
                        .candidate
                        .as_ref()
                        .map_or(0, |point| point.state.capacity() + point.rates.capacity())
                    + self
                        .assessment
                        .as_ref()
                        .map_or(0, |assessment| assessment.residual.capacity()))
    }
}
/// Compute consistent state/rates at one original time using the caller's original scope.
#[cfg(feature = "idas")]
pub fn initialize_consistent(
    oracle: &mut dyn Oracle,
    parameters: &[f64],
    request: &ConsistentInitialization,
    execution: crate::solve::Execution,
) -> Result<ConsistentStateReport, ProblemError> {
    idas::initialize_consistent(oracle, parameters, request, execution)
}
/// The native IDAS root direction of an event's guard crossing (`IDASetRootDirection`).
#[cfg_attr(
    not(feature = "idas"),
    expect(dead_code, reason = "the native root directions exist only with IDAS")
)]
pub(crate) const fn root_direction(direction: EventDirection) -> i32 {
    match direction {
        EventDirection::Either => 0,
        EventDirection::Rising => 1,
        EventDirection::Falling => -1,
    }
}

/// Dispatch only after checking the complete integration requirements. `parameters` is
/// the integration vector of [`Profile::integration_width`] values: the unscheduled
/// contract parameters, then every scheduled input's interval values.
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
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    integrate_with_progress_observed(oracle, profile, parameters, cancel, progress, &snapshot)
}
/// Execute against the immutable observation used for contextual routing.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub fn integrate_with_progress_observed(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    cancel: Cancellation,
    progress: Arc<crate::solve::Progress>,
    snapshot: &crate::execution::Snapshot,
) -> Result<Report, ProblemError> {
    let resolved = profile.resolve_for(
        oracle.contract(),
        parameters,
        snapshot,
        DynamicDemand::Base,
        oracle.contract().derivatives,
        &[],
    )?;
    let profile = &resolved;
    snapshot.validate_for(crate::execution::adapter(
        match profile.resolved_method()? {
            Method::Diffsol => crate::solve::Backend::Diffsol,
            Method::Idas => crate::solve::Backend::Idas,
            Method::Auto => return Err(ProblemError::internal("unresolved dynamic method")),
        },
    ))?;
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

/// A functional's gradient and the forward trajectory it was taken on (ADR-0110 item 3).
#[derive(Debug)]
pub struct Gradient {
    /// The forward pass, whose samples carry values without sensitivities. A backward pass
    /// that fails records its termination and cause here; the forward samples remain.
    pub report: Report,
    /// dJ/dp over the integration parameters, present when both passes completed.
    pub gradient: Option<Vec<f64>>,
    /// The second-order route's d²J/dpᵢdpⱼ over the requested directions, symmetric,
    /// present when both passes completed (ADR-0110 item 4); absent on the first-order
    /// route.
    pub hessian: Option<faer::Mat<f64>>,
    /// Forward checkpoints the native library actually stored.
    pub checkpoints: usize,
    /// The checkpoint estimate charged against the caller's allowance.
    pub reserved_bytes: usize,
}
/// The cotangent of a scalar functional J of the sampled outputs, from the completed
/// forward report: `dJ/d outputs[o]` at sample `i` in `i * outputs + o`.
pub type Cotangent<'c> = &'c mut dyn FnMut(&Report) -> Result<Vec<f64>, ProblemError>;

/// The gradient of a scalar functional of the sampled outputs with respect to the
/// integration parameters, by checkpointed adjoint sensitivities. The profile requests
/// [`DynamicSensitivity::Adjoint`]; the checkpoint estimate ([`Profile::checkpoint_bytes`])
/// must fit `memory` before any native work. The forward pass runs once, `cotangent` reads
/// its samples, and the backward pass carries the adjoint state across scheduled changes.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub fn gradient(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    memory: usize,
) -> Result<Gradient, ProblemError> {
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    gradient_observed(
        oracle, profile, parameters, cotangent, cancel, memory, &snapshot,
    )
}
/// Execute against the immutable observation used for contextual routing.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub fn gradient_observed(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    memory: usize,
    snapshot: &crate::execution::Snapshot,
) -> Result<Gradient, ProblemError> {
    let resolved = profile.resolve_for(
        oracle.contract(),
        parameters,
        snapshot,
        DynamicDemand::Base,
        oracle.contract().derivatives,
        &[],
    )?;
    let profile = &resolved;
    snapshot.validate_for(crate::execution::adapter(
        match profile.resolved_method()? {
            Method::Diffsol => crate::solve::Backend::Diffsol,
            Method::Idas => crate::solve::Backend::Idas,
            Method::Auto => return Err(ProblemError::internal("unresolved dynamic method")),
        },
    ))?;
    profile.validate(oracle.contract(), parameters)?;
    if profile.sensitivity != DynamicSensitivity::Adjoint {
        return Err(contract("a gradient needs the adjoint sensitivity profile"));
    }
    let reserved = profile.checkpoint_bytes(oracle.contract())?;
    if reserved > memory {
        return Err(ProblemError::memory(format!(
            "adjoint checkpoint estimate of {reserved} bytes exceeds the {memory}-byte allowance"
        )));
    }
    let progress = Arc::new(crate::solve::Progress::new(256));
    match profile.resolved_method()? {
        #[cfg(feature = "diffsol")]
        Method::Diffsol => {
            integrator::gradient(oracle, profile, parameters, cotangent, cancel, progress)
        }
        #[cfg(feature = "idas")]
        Method::Idas => idas::gradient(oracle, profile, parameters, cotangent, cancel, progress),
        _ => Err(ProblemError::unsupported(
            "requested dynamic backend is not linked",
        )),
    }
    .map(|mut g| {
        g.reserved_bytes = reserved;
        g
    })
}

/// The gradient and the exact Hessian of a scalar functional of the sampled outputs with
/// respect to the integration parameters in `directions`, by forward-over-adjoint
/// second-order sensitivities on IDAS (ADR-0110 item 4): the forward pass integrates the
/// state sensitivities with its checkpoints, and one backward problem of size 2n per
/// direction integrates the adjoint together with its tangent along that direction. The
/// cotangent `c` is held fixed, so the Hessian is `Σᵢ cᵢ·∇²yᵢ`; a least-squares caller adds
/// its Gauss–Newton part. The profile requests [`DynamicSensitivity::Adjoint`] and its
/// limits; the oracle declares second derivatives; the method resolves to IDAS, because
/// Diffsol has no second-order adjoint; and the estimate of the checkpoints with their
/// sensitivities and of the backward problems must fit `memory` before native work.
#[cfg(feature = "idas")]
pub fn hessian(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    directions: &[usize],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    memory: usize,
) -> Result<Gradient, ProblemError> {
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    hessian_observed(
        oracle, profile, parameters, directions, cotangent, cancel, memory, &snapshot,
    )
}
/// Execute against the immutable observation used for contextual routing.
#[cfg(feature = "idas")]
#[expect(
    clippy::too_many_arguments,
    reason = "the second-order adjoint boundary binds oracle/profile/parameters, directions/cotangent, cancellation, admitted memory and the retained runtime snapshot"
)]
pub fn hessian_observed(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    directions: &[usize],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    memory: usize,
    snapshot: &crate::execution::Snapshot,
) -> Result<Gradient, ProblemError> {
    let resolved = profile.resolve_for(
        oracle.contract(),
        parameters,
        snapshot,
        DynamicDemand::ExactHessian,
        oracle.contract().derivatives,
        directions,
    )?;
    let profile = &resolved;
    snapshot.validate_for(crate::execution::adapter(crate::solve::Backend::Idas))?;
    profile.validate(oracle.contract(), parameters)?;
    let reserved = profile.admit_second_order(oracle.contract(), directions)?;
    if reserved > memory {
        return Err(ProblemError::memory(format!(
            "second-order adjoint estimate of {reserved} bytes exceeds the {memory}-byte allowance"
        )));
    }
    let progress = Arc::new(crate::solve::Progress::new(256));
    idas::hessian(
        oracle, profile, parameters, directions, cotangent, cancel, progress,
    )
    .map(|mut g| {
        g.reserved_bytes = reserved;
        g
    })
}

/// The jump of the adjoint at an observed sample, shared by both adjoint backends
/// (ADR-0110 item 3). With `c` the cotangent of the sample's outputs `y = g(x, p)`, the
/// differential adjoint gains `dJ/dx_d` and the gradient gains `dJ/dp` over the contract
/// parameters, both total derivatives through the algebraic states, which the index-1
/// constraint `F_a(x_d, x_a, p) = 0` determines: with `F_aaᵀ w = g_aᵀc`,
/// `dJ/dx_d = g_dᵀc − F_adᵀw` and `dJ/dp = g_pᵀc − F_apᵀw`. The algebraic adjoint has no
/// jump of its own; the integrator's consistent initialization recomputes it. `output` and
/// `rhs` are the raw partials over the state followed by the contract parameters; `rhs` is
/// needed only with algebraic states.
#[cfg(feature = "diffsol")]
pub(crate) fn sample_jump(
    differential: &[bool],
    output: faer::sparse::SparseColMatRef<'_, usize, f64>,
    rhs: Option<faer::sparse::SparseColMatRef<'_, usize, f64>>,
    cotangent: &[f64],
) -> Result<(Vec<f64>, Vec<f64>), ProblemError> {
    if output.nrows() != cotangent.len() || output.ncols() < differential.len() {
        return Err(ProblemError::internal("adjoint jump extent"));
    }
    let eliminated = eliminate(differential, rhs, transposed_product(output, cotangent))?;
    Ok((eliminated.state, eliminated.parameters))
}
/// A total derivative over the state followed by the contract parameters after the
/// algebraic states are eliminated through the index-1 constraint.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) struct Eliminated {
    /// Over the state: the differential part; zero on algebraic states.
    pub(crate) state: Vec<f64>,
    /// Over the contract parameters.
    pub(crate) parameters: Vec<f64>,
    /// The constraint multipliers `w` of `F_aaᵀ w = b_a`, scattered over the algebraic rows
    /// of the residual function and zero on its differential rows.
    #[cfg(feature = "idas")]
    pub(crate) multipliers: Vec<f64>,
}
/// `Aᵀv` over a sparse matrix's columns.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) fn transposed_product(
    matrix: faer::sparse::SparseColMatRef<'_, usize, f64>,
    v: &[f64],
) -> Vec<f64> {
    let mut out = vec![0.0; matrix.ncols()];
    faer::sparse::linalg::matmul::sparse_dense_matmul(
        faer::MatMut::from_column_major_slice_mut(&mut out, matrix.ncols(), 1),
        faer::Accum::Replace,
        matrix.transpose(),
        faer::MatRef::from_column_major_slice(v, v.len(), 1),
        1.0,
        faer::Par::Seq,
    );
    out
}
/// Eliminate the algebraic states from `b`, a derivative over the state followed by the
/// contract parameters: with `F_aaᵀ w = b_a`, the result is `b − F_zᵀw` with its algebraic
/// entries zero. `rhs` holds the residual function's raw partials and is needed only with
/// algebraic states. The first-order sample jump eliminates `g_zᵀc`; the second-order
/// jump eliminates its tangent (ADR-0110 item 4).
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) fn eliminate(
    differential: &[bool],
    rhs: Option<faer::sparse::SparseColMatRef<'_, usize, f64>>,
    mut jump: Vec<f64>,
) -> Result<Eliminated, ProblemError> {
    use faer::linalg::solvers::Solve;
    let n = differential.len();
    let width = jump.len();
    if width < n {
        return Err(ProblemError::internal("adjoint jump extent"));
    }
    let mut multipliers = vec![0.0; n];
    let algebraic = (0..n).filter(|i| !differential[*i]).collect::<Vec<_>>();
    if !algebraic.is_empty() {
        let f = rhs.ok_or_else(|| ProblemError::internal("adjoint jump constraint partials"))?;
        if f.nrows() != n || f.ncols() != width {
            return Err(ProblemError::internal("adjoint jump constraint extent"));
        }
        let mut slot = vec![None; n];
        for (k, i) in algebraic.iter().enumerate() {
            slot[*i] = Some(k);
        }
        // F_aaᵀ: the constraint rows' partials in the algebraic columns, transposed.
        let mut triplets = Vec::new();
        for column in &algebraic {
            for k in f.col_range(*column) {
                if let (Some(i), Some(j)) = (slot[f.row_idx()[k]], slot[*column]) {
                    triplets.push(faer::sparse::Triplet::new(j, i, f.val()[k]));
                }
            }
        }
        let na = algebraic.len();
        let transposed = faer::sparse::SparseColMat::try_new_from_triplets(na, na, &triplets)
            .map_err(|e| ProblemError::memory(format!("adjoint jump constraint block: {e:?}")))?;
        let symbolic = faer::sparse::linalg::solvers::SymbolicLu::try_new(transposed.symbolic())
            .map_err(|e| ProblemError::numerical(format!("adjoint jump constraint LU: {e:?}")))?;
        let lu =
            faer::sparse::linalg::solvers::Lu::try_new_with_symbolic(symbolic, transposed.as_ref())
                .map_err(|e| {
                    ProblemError::numerical(format!("adjoint jump constraint LU: {e:?}"))
                })?;
        let mut w = algebraic.iter().map(|i| jump[*i]).collect::<Vec<_>>();
        lu.solve_in_place(faer::MatMut::from_column_major_slice_mut(&mut w, na, 1));
        if w.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "singular algebraic block at an observed sample",
            ));
        }
        for (k, i) in algebraic.iter().enumerate() {
            multipliers[*i] = w[k];
        }
        for (total, correction) in jump.iter_mut().zip(transposed_product(f, &multipliers)) {
            *total -= correction;
        }
        for i in &algebraic {
            jump[*i] = 0.0;
        }
    }
    if jump.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite adjoint jump"));
    }
    let parameters = jump.split_off(n);
    Ok(Eliminated {
        state: jump,
        parameters,
        #[cfg(feature = "idas")]
        multipliers,
    })
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
    /// Physical conserved inventories, in balance order, using the active mode definition.
    Inventory,
    /// Physical permitted transfers for one event, evaluated in the pre-event state and mode.
    /// Values follow balance order; undeclared subjects must return zero.
    Transfer(usize),
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
    pub direction: EventDirection,
}
/// One conserved subject with independently compiled physical inventory and original flux.
#[derive(Clone, Debug)]
pub struct Balance {
    /// Stable conserved subject identity across modes.
    pub id: SemanticId,
    /// Authored inventory expression identity; its value is evaluated in balance order.
    pub inventory: SemanticId,
    /// Original signed flux quadrature identity, independent of the solved rates.
    pub flux: SemanticId,
    /// Absolute physical conserved quantity closure tolerance.
    pub tolerance: f64,
    /// Authored nonterminal events permitted to transfer this subject.
    pub transfers: BTreeSet<SemanticId>,
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
    /// Empty, or the sign each state keeps, in state order: derived from the authored
    /// bounds and applied by IDAS (`IDASetConstraints`) to keep its steps in the domain.
    /// The bounds' guard remains the validity authority; Diffsol has no such control
    /// (ADR-0119 Outcome 4).
    pub signs: Vec<StateSign>,
    /// The exact derivative order the oracle's functions provide: first-order partials
    /// for integration and sensitivities, second order for [`Oracle::weighted_hessian`]
    /// and the exact transient Hessian (ADR-0110 item 4).
    pub derivatives: pse_kernels::DerivativeOrder,
}
impl Contract {
    /// Validate finite layout and the concrete supported mass/event profile.
    pub fn validate(&self) -> Result<(), ProblemError> {
        let unique = |ids: &[SemanticId]| ids.iter().collect::<BTreeSet<_>>().len() == ids.len();
        if self.derivatives < pse_kernels::DerivativeOrder::First {
            return Err(contract(
                "dynamic functions need at least first-order partials",
            ));
        }
        if !unique(&self.quadratures)
            || !unique(&self.balances.iter().map(|b| b.id).collect::<Vec<_>>())
            || self.balances.iter().any(|b| {
                !self.quadratures.contains(&b.flux)
                    || !positive(b.tolerance)
                    || b.transfers.iter().any(|id| {
                        !self
                            .events
                            .iter()
                            .flatten()
                            .any(|e| e.id == *id && !e.terminal)
                    })
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
            || !(self.signs.is_empty() || self.signs.len() == self.states.len())
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
/// Explicit final-result obligation, independent of the requested output grid.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct EndpointRequirement {
    /// Registry-owned completion policy.
    pub kind: pse_model::generated::enums::EndpointPolicy,
    /// Allowed terminal guard identity, required only for declared-event completion.
    pub event: Option<SemanticId>,
}
impl Default for EndpointRequirement {
    fn default() -> Self {
        Self {
            kind: pse_model::generated::enums::EndpointPolicy::FixedHorizon,
            event: None,
        }
    }
}
/// Coverage retains final completion and required downstream observations separately.
#[derive(Clone, Debug, PartialEq)]
pub struct EndpointAssessment {
    /// The retained endpoint satisfies the admitted completion requirement.
    pub satisfied: bool,
    /// Every requested sample through the actual endpoint exists, in order.
    pub prefix_complete: bool,
    /// Requested observations after the endpoint, still required by fixed-domain consumers.
    pub missing_observations: Vec<f64>,
}
/// Finite integration policy. Tolerances apply to the normalized state coordinates.
#[derive(Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
    /// Declared completion event; omitted means the full fixed horizon.
    #[serde(default)]
    pub endpoint: EndpointRequirement,
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
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub time_limit: Duration,
    /// Maximum retained scalar cells, including sensitivities and event states.
    pub max_cells: usize,
    /// Parameter derivatives: none, forward sensitivities of every sample, or adjoint
    /// gradients of one functional of the sampled outputs through `gradient`; none when
    /// absent.
    #[serde(default = "no_sensitivity")]
    pub sensitivity: DynamicSensitivity,
    /// Checkpointing of the adjoint backward pass; used only by adjoint gradients.
    #[serde(default)]
    pub adjoint: AdjointSettings,
    /// Positive characteristic scales in the contract's parameter order, also used for
    /// sensitivity tolerances; every interval of a scheduled input takes its parameter's.
    pub parameter_scales: Vec<f64>,
    /// Scheduled inputs: a contract parameter that takes one integration value per
    /// schedule interval, each with its own live sensitivity (I6).
    #[serde(default)]
    pub schedule: Vec<ScheduledInput>,
    /// Typed Diffsol scheme and linear solver.
    #[serde(default)]
    pub diffsol: DiffsolSettings,
    /// Typed IDAS linear solver, sensitivity corrector and start.
    #[serde(default)]
    pub idas: IdasSettings,
    /// Library-owned initialization controls, transported in every profile.
    #[serde(with = "initial_options")]
    #[schemars(with = "initial_options::Remote")]
    pub initialization: Arc<diffsol::InitialConditionSolverOptions<f64>>,
    /// Native BDF/nonlinear/error-control settings.
    #[serde(with = "ode_options")]
    #[schemars(with = "ode_options::Remote")]
    pub native: Arc<diffsol::OdeSolverOptions<f64>>,
}

#[cfg(test)]
mod duration_schema_unit {
    use super::*;

    #[test]
    fn dynamics_profile_retains_library_controls_in_every_build() {
        let mut profile = Profile::default();
        Arc::get_mut(&mut profile.initialization)
            .unwrap()
            .max_newton_iterations = 17;
        Arc::get_mut(&mut profile.native)
            .unwrap()
            .nonlinear_solver_tolerance = 0.0123;
        let encoded = serde_json::to_value(&profile).unwrap();
        assert_eq!(encoded["initialization"]["max_newton_iterations"], 17);
        assert_eq!(encoded["native"]["nonlinear_solver_tolerance"], 0.0123);
        let decoded: Profile = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(serde_json::to_value(&decoded).unwrap(), encoded);
        assert_eq!(settings_identity(&decoded), settings_identity(&profile));
        assert_ne!(
            settings_identity(&decoded),
            settings_identity(&Profile::default())
        );

        let schema = schemars::schema_for!(Profile).to_value();
        let required = schema["required"].as_array().unwrap();
        for field in ["initialization", "native"] {
            assert!(required.contains(&serde_json::json!(field)));
            let mut incomplete = encoded.clone();
            incomplete.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<Profile>(incomplete).is_err());
        }
        assert_eq!(
            schema["properties"]["initialization"]["$ref"],
            "#/$defs/DiffsolInitialConditionOptions"
        );
        assert_eq!(
            schema["properties"]["native"]["$ref"],
            "#/$defs/DiffsolOdeSolverOptions"
        );
    }

    #[test]
    fn dynamics_profile_duration_schema_matches_closed_serde_representation() {
        let profile = Profile {
            time_limit: Duration::new(7, 123),
            ..Profile::default()
        };
        let encoded = serde_json::to_value(&profile).unwrap();
        assert_eq!(
            encoded["time_limit"],
            serde_json::json!({"secs": 7, "nanos": 123})
        );
        let decoded: Profile = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(decoded.time_limit, profile.time_limit);
        let mut extra = encoded.clone();
        extra["time_limit"]["seconds"] = serde_json::json!(8);
        assert!(serde_json::from_value::<Profile>(extra).is_err());
        let mut overflow = encoded;
        overflow["time_limit"]["secs"] = serde_json::json!(u64::MAX);
        overflow["time_limit"]["nanos"] = serde_json::json!(u32::MAX);
        assert!(serde_json::from_value::<Profile>(overflow).is_err());
        let schema = schemars::schema_for!(Profile).to_value();
        assert_eq!(
            schema["properties"]["time_limit"]["$ref"],
            "#/$defs/ClosedDuration"
        );
        assert_eq!(
            schema["$defs"]["ClosedDuration"]["additionalProperties"],
            false
        );
        assert_eq!(
            schema["$defs"]["ClosedDuration"]["required"],
            serde_json::json!(["secs", "nanos"])
        );
    }
}

/// One contract parameter held piecewise constant: it takes a new value at each change
/// time. The integration parameter vector holds one value per interval, `times.len() + 1`
/// of them, and a sensitivity column for each, so sensitivities cross every change
/// (ADR-0119, I6).
///
/// ```
/// use pse_backend_native::dynamics::{Profile, ScheduledInput};
///
/// // Two contract parameters; the second changes at t = 0.5.
/// let profile = Profile {
///     schedule: vec![ScheduledInput { parameter: 1, times: vec![0.5] }],
///     ..Profile::default()
/// };
/// // The static parameter comes first, then one value per interval.
/// assert_eq!(profile.integration_width(2), 3);
/// assert_eq!(profile.integration_parameters(&[4.0, 7.0]), vec![4.0, 7.0, 7.0]);
/// assert_eq!(profile.parameters_at(&[4.0, 7.0, 9.0], 0.25), vec![4.0, 7.0]);
/// // A change takes effect at its time.
/// assert_eq!(profile.parameters_at(&[4.0, 7.0, 9.0], 0.5), vec![4.0, 9.0]);
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScheduledInput {
    /// Position of the input in the contract's parameter order.
    pub parameter: usize,
    /// Strictly increasing change times after the start and up to the end; a change at
    /// the end is observed by the final sample only.
    pub times: Vec<f64>,
}
/// Mandatory dynamic consumer demand, separate from optional post-solve analyses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynamicDemand {
    /// The profile's base integration/sensitivity contract.
    Base,
    /// Exact transient Hessian over the named integration directions.
    ExactHessian,
}
/// One method's original typed contextual assessment.
#[derive(Clone, Debug)]
pub struct DynamicCandidate {
    /// Concrete method being assessed.
    pub method: Method,
    /// Every retained settings/representation refusal.
    pub causes: Vec<Arc<ProblemError>>,
    /// Required selected derivative order not yet prepared.
    pub artifacts: Vec<crate::routing::ArtifactDemand>,
    /// Readiness after the whole requested dynamic contract is assessed.
    pub state: crate::routing::AssessmentState,
}
/// Deterministic dynamic selection against one immutable observation.
#[derive(Clone, Debug)]
pub struct DynamicDecision {
    /// The authored explicit or automatic method choice.
    pub requested: Method,
    /// Selected concrete method, absent on contextual refusal.
    pub selected: Option<Method>,
    /// Both methods for Auto; only the requested method for explicit selection.
    pub candidates: Vec<DynamicCandidate>,
    /// Exact consumed build/runtime observation.
    pub snapshot: ContentHash,
}
impl DynamicDecision {
    /// A selected scientific method whose required artifacts the caller must prepare.
    pub fn candidate(&self) -> Result<Method, ProblemError> {
        self.selected
            .ok_or_else(|| ProblemError::DynamicRouteRefused(Box::new(self.clone())))
    }
    /// Final readiness; pending kernels never enter a native attempt.
    pub fn ready(&self) -> Result<Method, ProblemError> {
        self.selected
            .filter(|method| {
                self.candidates.iter().any(|candidate| {
                    candidate.method == *method
                        && candidate.state == crate::routing::AssessmentState::Ready
                })
            })
            .ok_or_else(|| ProblemError::DynamicRouteRefused(Box::new(self.clone())))
    }
    /// Owned retained diagnostic and pending-demand extent.
    pub fn retained_bytes(&self) -> usize {
        self.candidates.capacity() * size_of::<DynamicCandidate>()
            + self
                .candidates
                .iter()
                .map(|candidate| {
                    candidate.causes.capacity() * size_of::<Arc<ProblemError>>()
                        + candidate
                            .causes
                            .iter()
                            .map(|cause| cause.retained_bytes())
                            .sum::<usize>()
                        + candidate.artifacts.capacity()
                            * size_of::<crate::routing::ArtifactDemand>()
                })
                .sum::<usize>()
    }
}
impl Profile {
    /// The integration parameter vector's length for `parameters` contract parameters:
    /// the unscheduled parameters, then every interval of every scheduled input.
    pub fn integration_width(&self, parameters: usize) -> usize {
        parameters.saturating_sub(self.schedule.len())
            + self
                .schedule
                .iter()
                .map(|s| s.times.len() + 1)
                .sum::<usize>()
    }
    /// The integration column of every contract parameter while time `t` is in effect.
    /// Unscheduled parameters take the leading columns in contract order; a scheduled
    /// input's interval columns follow in schedule order, and a change applies from its
    /// own time on.
    pub fn columns_at(&self, parameters: usize, t: f64) -> Vec<usize> {
        let mut columns = vec![0; parameters];
        let mut next = 0;
        for (k, column) in columns.iter_mut().enumerate() {
            if !self.schedule.iter().any(|s| s.parameter == k) {
                *column = next;
                next += 1;
            }
        }
        for s in &self.schedule {
            if let Some(column) = columns.get_mut(s.parameter) {
                *column = next + s.times.iter().filter(|c| **c <= t).count();
            }
            next += s.times.len() + 1;
        }
        columns
    }
    /// The contract parameter values in effect at `t`, from an integration vector.
    pub fn parameters_at(&self, integration: &[f64], t: f64) -> Vec<f64> {
        let np = self
            .contract_width(integration.len())
            .unwrap_or(integration.len());
        self.columns_at(np, t)
            .into_iter()
            .map(|c| integration.get(c).copied().unwrap_or(f64::NAN))
            .collect()
    }
    /// The integration vector that holds every scheduled input at its contract value in
    /// each interval; callers then set the interval values they schedule.
    pub fn integration_parameters(&self, contract: &[f64]) -> Vec<f64> {
        let mut values = vec![0.0; self.integration_width(contract.len())];
        let first = self.columns_at(contract.len(), self.start);
        for (k, value) in contract.iter().enumerate() {
            let first = first[k];
            let intervals = self
                .schedule
                .iter()
                .find(|s| s.parameter == k)
                .map_or(1, |s| s.times.len() + 1);
            if let Some(slots) = values.get_mut(first..first + intervals) {
                slots.fill(*value);
            }
        }
        values
    }
    /// The contract parameter count whose integration vector has `integration` values.
    fn contract_width(&self, integration: usize) -> Option<usize> {
        let intervals = self
            .schedule
            .iter()
            .map(|s| s.times.len() + 1)
            .sum::<usize>();
        integration
            .checked_sub(intervals)
            .map(|n| n + self.schedule.len())
    }
    /// The profile of a window `[start, end]` of this horizon, sampled at `samples`: every
    /// scheduled input keeps the changes strictly inside the window, and one in effect at
    /// `start` becomes the window's first interval. A shooting window's integration vector
    /// holds, for each of its columns, the value of the horizon's column in effect there
    /// ([`Profile::columns_at`] of both at the same time).
    ///
    /// ```
    /// use pse_backend_native::dynamics::{Profile, ScheduledInput};
    ///
    /// let horizon = Profile {
    ///     end: 3.0,
    ///     schedule: vec![ScheduledInput { parameter: 0, times: vec![1.0, 2.0] }],
    ///     ..Profile::default()
    /// };
    /// let window = horizon.window(1.0, 3.0, vec![1.0, 3.0]);
    /// assert_eq!((window.start, window.end), (1.0, 3.0));
    /// assert_eq!(window.schedule, vec![ScheduledInput { parameter: 0, times: vec![2.0] }]);
    /// // Without a change inside, the input is constant over the window.
    /// assert!(horizon.window(0.0, 1.0, vec![1.0]).schedule.is_empty());
    /// ```
    pub fn window(&self, start: f64, end: f64, samples: Vec<f64>) -> Profile {
        let mut window = self.clone();
        window.start = start;
        window.end = end;
        window.samples = samples;
        window.schedule = self
            .schedule
            .iter()
            .map(|s| ScheduledInput {
                parameter: s.parameter,
                times: s
                    .times
                    .iter()
                    .copied()
                    .filter(|t| *t > start && *t < end)
                    .collect(),
            })
            .filter(|s| !s.times.is_empty())
            .collect();
        window
    }
    /// Every distinct change time in increasing order: the segment boundaries.
    pub(crate) fn boundaries(&self) -> Vec<f64> {
        let mut times = self
            .schedule
            .iter()
            .flat_map(|s| s.times.iter().copied())
            .collect::<Vec<_>>();
        times.sort_by(f64::total_cmp);
        times.dedup();
        times
    }
    /// Assess the complete requested contract without acquiring an evaluator or worker.
    /// Resource/cancellation/infrastructure failures stop assessment, never select another method.
    pub fn assess_method(
        &self,
        c: &Contract,
        parameters: &[f64],
        snapshot: &crate::execution::Snapshot,
        demand: DynamicDemand,
        prepared_order: pse_kernels::DerivativeOrder,
        directions: &[usize],
    ) -> Result<DynamicDecision, ProblemError> {
        use crate::{
            routing::{ArtifactDemand, AssessmentState},
            solve::Backend,
        };
        let methods: &[Method] = match self.method {
            Method::Auto => &[Method::Diffsol, Method::Idas],
            Method::Diffsol => &[Method::Diffsol],
            Method::Idas => &[Method::Idas],
        };
        let mut candidates = Vec::new();
        for &method in methods {
            let mut profile = self.clone();
            profile.method = method;
            let backend = if method == Method::Diffsol {
                Backend::Diffsol
            } else {
                Backend::Idas
            };
            let mut causes = Vec::new();
            if !snapshot.linked(backend) {
                causes.push(Arc::new(ProblemError::Unavailable {
                    backend,
                    alternatives: vec![],
                }));
            }
            causes.extend(profile.method_refusals(c).into_iter().map(Arc::new));
            match profile.validate_common(c, parameters) {
                Ok(_) => {}
                Err(
                    cause @ (ProblemError::Limit { .. }
                    | ProblemError::Cancelled
                    | ProblemError::Internal(_)),
                ) => return Err(cause),
                Err(cause) => causes.push(Arc::new(cause)),
            }
            let order = if demand == DynamicDemand::ExactHessian {
                match profile.admit_second_order(c, directions) {
                    Ok(_) => {}
                    Err(
                        cause @ (ProblemError::Limit { .. }
                        | ProblemError::Cancelled
                        | ProblemError::Internal(_)),
                    ) => return Err(cause),
                    Err(cause) => causes.push(Arc::new(cause)),
                }
                pse_kernels::DerivativeOrder::Second
            } else {
                pse_kernels::DerivativeOrder::First
            };
            let artifacts = if c.derivatives >= order && prepared_order < order {
                vec![ArtifactDemand::Derivatives(order)]
            } else {
                vec![]
            };
            let state = if !causes.is_empty() {
                AssessmentState::Refused
            } else if !artifacts.is_empty() {
                AssessmentState::SupportedPendingArtifacts
            } else {
                AssessmentState::Ready
            };
            candidates.push(DynamicCandidate {
                method,
                causes,
                artifacts,
                state,
            });
        }
        // Diffsol remains the preferred Auto method when its whole mandatory contract is supported.
        let selected = candidates
            .iter()
            .find(|candidate| candidate.state != AssessmentState::Refused)
            .map(|candidate| candidate.method);
        Ok(DynamicDecision {
            requested: self.method,
            selected,
            candidates,
            snapshot: snapshot.identity(),
        })
    }
    /// Materialize the scientific selection as an explicit immutable profile.
    pub fn resolve_for(
        &self,
        c: &Contract,
        parameters: &[f64],
        snapshot: &crate::execution::Snapshot,
        demand: DynamicDemand,
        prepared_order: pse_kernels::DerivativeOrder,
        directions: &[usize],
    ) -> Result<Self, ProblemError> {
        let decision =
            self.assess_method(c, parameters, snapshot, demand, prepared_order, directions)?;
        let mut profile = self.clone();
        profile.method = decision.candidate()?;
        Ok(profile)
    }
    /// Read the concrete admitted method. Auto requires whole-contract assessment first.
    pub fn resolved_method(&self) -> Result<Method, ProblemError> {
        match self.method {
            Method::Auto => Err(contract(
                "automatic dynamic method requires contextual contract assessment",
            )),
            method => Ok(method),
        }
    }
    /// Forward sensitivities are integrated with every sample.
    pub fn forward(&self) -> bool {
        self.sensitivity == DynamicSensitivity::Forward
    }
    /// The adjoint route's profile limits (ADR-0110 items 3 and 6), refused before native
    /// work: the functional observes sampled outputs only, so declared quadratures need
    /// forward sensitivities; no event is crossed, because Diffsol has no mass-matrix reset
    /// adjoint and keeps the reset metadata between its checkpoint segments private, and
    /// IDAS has no reset sensitivities; the start keeps its requested differential values;
    /// every scheduled change precedes the end; and every scheduled segment stores at least
    /// its two end checkpoints.
    fn admit_adjoint(&self, c: &Contract) -> Result<(), ProblemError> {
        if c.events.iter().any(|e| !e.is_empty()) {
            return Err(ProblemError::unsupported(
                "adjoint gradients do not cross events or resets; request forward sensitivities",
            ));
        }
        if !c.quadratures.is_empty() {
            return Err(ProblemError::unsupported(
                "adjoint gradients observe sampled outputs only; declared quadratures need forward sensitivities",
            ));
        }
        // A steady start's values solve the model at its parameters; their partials would
        // need that solve's adjoint, which neither backend's initialization provides.
        if self.idas.initialization == IdasInitialization::SteadyStates {
            return Err(ProblemError::unsupported(
                "adjoint gradients need the algebraic-and-rates initialization",
            ));
        }
        // A change at the end is observed by the final sample alone: that single-point
        // segment has no interval for a backward pass.
        if self.boundaries().last().is_some_and(|t| *t >= self.end) {
            return Err(contract(
                "adjoint gradients need every scheduled change before the end",
            ));
        }
        let segments = self.segments();
        if self.adjoint.max_checkpoints.into_inner() < segments.saturating_mul(2) {
            return Err(contract(
                "adjoint max_checkpoints below two per scheduled segment",
            ));
        }
        Ok(())
    }
    /// The number of scheduled segments: one more than the distinct change times.
    fn segments(&self) -> usize {
        self.boundaries().len() + 1
    }
    /// The estimated bytes of the adjoint forward checkpoints and their replay buffers,
    /// charged against the caller's allowance before native work. The count is the smaller
    /// of `max_checkpoints` and what `max_steps` can store; each Diffsol checkpoint is a
    /// method state bounded by a BDF difference table (the largest), each IDAS checkpoint
    /// its history array, and the replay holds `steps_between_checkpoints` Hermite points
    /// per segment (two interpolants on Diffsol). Accounting, not an RSS claim.
    pub fn checkpoint_bytes(&self, c: &Contract) -> Result<usize, ProblemError> {
        let n = c.states.len();
        let steps = self.adjoint.steps_between_checkpoints.into_inner();
        let segments = self.segments();
        let (state, interpolants) = match self.resolved_method()? {
            Method::Idas => (6, 1),
            _ => (16, 2),
        };
        let checkpoints = segments
            .checked_mul(2)
            .and_then(|ends| (self.max_steps / steps).checked_add(ends))
            .map(|count| count.min(self.adjoint.max_checkpoints.into_inner()));
        let bytes = checkpoints
            .and_then(|count| count.checked_mul(n.checked_mul(state)?.checked_add(64)?))
            .and_then(|b| {
                b.checked_add(
                    segments
                        .checked_mul(interpolants)?
                        .checked_mul(steps.checked_add(2)?)?
                        .checked_mul(n.checked_mul(2)?.checked_add(1)?)?,
                )
            })
            .and_then(|cells| cells.checked_mul(size_of::<f64>()))
            .ok_or_else(|| ProblemError::memory("adjoint checkpoint extent overflow"))?;
        Ok(bytes)
    }
    /// The second-order route's admission (ADR-0110 item 4), before native work: the
    /// adjoint profile and its limits (checked by [`Profile::validate`]), IDAS, declared
    /// second derivatives, distinct integration columns as directions, and the sampled
    /// state and output sensitivities within the cell allowance. Returns the estimated
    /// bytes of the checkpoints with their stored sensitivities, the replay buffers and the
    /// backward problems, charged against the caller's allowance. Accounting, not an RSS
    /// claim.
    pub fn admit_second_order(
        &self,
        c: &Contract,
        directions: &[usize],
    ) -> Result<usize, ProblemError> {
        if self.sensitivity != DynamicSensitivity::Adjoint {
            return Err(contract(
                "an exact transient Hessian needs the adjoint sensitivity profile",
            ));
        }
        if self.resolved_method()? != Method::Idas {
            return Err(ProblemError::unsupported(
                "exact transient Hessians need IDAS forward-over-adjoint sensitivities; Diffsol has no second-order adjoint",
            ));
        }
        if c.derivatives < pse_kernels::DerivativeOrder::Second {
            return Err(ProblemError::unsupported(
                "an exact transient Hessian needs second derivatives of the dynamic functions",
            ));
        }
        let n = c.states.len();
        let width = self.integration_width(c.parameters.len());
        if directions.is_empty()
            || directions.iter().any(|d| *d >= width)
            || directions.iter().collect::<BTreeSet<_>>().len() != directions.len()
        {
            return Err(contract(
                "second-order directions are distinct integration columns",
            ));
        }
        let cells = self
            .samples
            .len()
            .checked_mul(c.outputs.len().saturating_add(n))
            .and_then(|v| v.checked_mul(width.checked_add(1)?))
            .and_then(|v| v.checked_add(self.max_events.checked_mul(n)?.checked_mul(2)?))
            .and_then(|v| {
                if c.balances.is_empty() {
                    return Some(v);
                }
                let points = self
                    .max_events
                    .checked_mul(3)?
                    .checked_add(self.samples.len())?
                    .checked_add(2)?;
                let width = c
                    .balances
                    .len()
                    .checked_mul(3)?
                    .checked_add(c.quadratures.len())?;
                points.checked_mul(width)?.checked_add(v)
            })
            .and_then(|v| {
                v.checked_add(
                    n.checked_add(c.outputs.len())?
                        .checked_add(c.quadratures.len())?
                        .checked_add(c.parameters.len().checked_mul(2)?)?,
                )
            })
            .ok_or_else(|| contract("dynamic result extent overflow"))?;
        if cells > self.max_cells {
            return Err(ProblemError::memory("dynamic result cell allowance"));
        }
        let steps = self.adjoint.steps_between_checkpoints.into_inner();
        let segments = self.segments();
        // Each IDAS checkpoint holds its history array for the state and every state
        // sensitivity; each Hermite point the values and rates of both.
        let stored = width.checked_add(1).and_then(|w| w.checked_mul(n));
        let checkpoints = segments
            .checked_mul(2)
            .and_then(|ends| (self.max_steps / steps).checked_add(ends))
            .map(|count| count.min(self.adjoint.max_checkpoints.into_inner()));
        // Each backward problem integrates 2n adjoint values and 2·width quadratures.
        let backward = n
            .checked_mul(2 * 24)
            .and_then(|a| a.checked_add(width.checked_mul(2 * 8)?))
            .and_then(|per| per.checked_mul(directions.len()));
        let bytes = checkpoints
            .zip(stored)
            .and_then(|(count, stored)| count.checked_mul(stored.checked_mul(6)?.checked_add(64)?))
            .and_then(|b| {
                b.checked_add(
                    segments
                        .checked_mul(steps.checked_add(2)?)?
                        .checked_mul(stored?.checked_mul(2)?.checked_add(1)?)?,
                )
            })
            .and_then(|b| b.checked_add(backward?))
            .and_then(|cells| cells.checked_mul(size_of::<f64>()))
            .ok_or_else(|| ProblemError::memory("second-order adjoint extent overflow"))?;
        Ok(bytes)
    }
    fn method_refusals(&self, c: &Contract) -> Vec<ProblemError> {
        let mut causes = Vec::new();
        match self.method {
            Method::Idas => {
                if let Err(error) = self.idas.initial_conditions.validate_for(
                    self.idas.initialization,
                    c.signs.iter().any(|sign| *sign != StateSign::Free),
                ) {
                    causes.push(error);
                }
                let changed = settings_identity(self) != settings_identity(&Self::default());
                if changed {
                    causes.push(contract(
                        "Diffsol-specific controls cannot be applied to IDAS",
                    ));
                }
                if self.forward() && c.events.iter().any(|events| !events.is_empty()) {
                    causes.push(ProblemError::unsupported("IDAS forward sensitivities do not cross events; Diffsol owns reset sensitivities"));
                }
                if matches!(
                    self.idas.linear,
                    IdasLinear::Spgmr {
                        preconditioner: crate::solve::Preconditioner::BlockFactor,
                        ..
                    } | IdasLinear::Spfgmr {
                        preconditioner: crate::solve::Preconditioner::BlockFactor,
                        ..
                    }
                ) {
                    causes.push(ProblemError::unsupported(
                        "IDAS block factor preconditioning is not supplied",
                    ));
                }
                if match self.idas.linear {
                    IdasLinear::Klu => false,
                    IdasLinear::Spgmr { dimension, .. } | IdasLinear::Spfgmr { dimension, .. } => {
                        i32::try_from(dimension.into_inner()).is_err()
                    }
                } {
                    causes.push(contract("invalid IDAS Krylov control"));
                }
            }
            Method::Diffsol => {
                if self.trial_failures == TrialPolicy::Recoverable {
                    causes.push(ProblemError::unsupported(
                        "Diffsol cannot recover typed trial failures",
                    ));
                }
                if self.idas != IdasSettings::default() {
                    causes.push(contract(
                        "IDAS-specific controls cannot be applied to Diffsol",
                    ));
                }
                if c.events
                    .iter()
                    .flatten()
                    .any(|event| event.direction != EventDirection::Either)
                {
                    causes.push(ProblemError::unsupported(
                        "Diffsol detects every guard sign change; a directional event needs IDAS",
                    ));
                }
                if self.diffsol.method == DiffsolMethod::Tsit45 {
                    if c.differential.contains(&false) {
                        causes.push(ProblemError::unsupported("explicit tsit45 integrates mass-free ODEs only; algebraic states need an implicit scheme"));
                    }
                    if self.diffsol.linear != DiffsolSettings::default().linear {
                        causes.push(contract("explicit tsit45 has no Newton linear solver"));
                    }
                }
            }
            Method::Auto => causes.push(ProblemError::internal("unresolved dynamic method")),
        }
        causes
    }
    /// Validate before allocation or native construction; arithmetic overflow is a refusal.
    pub fn validate(&self, c: &Contract, p: &[f64]) -> Result<usize, ProblemError> {
        if let Some(cause) = self.method_refusals(c).into_iter().next() {
            return Err(cause);
        }
        self.validate_common(c, p)
    }
    fn validate_common(&self, c: &Contract, p: &[f64]) -> Result<usize, ProblemError> {
        use pse_model::generated::enums::EndpointPolicy;
        match self.endpoint.kind {
            EndpointPolicy::FixedHorizon if self.endpoint.event.is_none() => {}
            EndpointPolicy::DeclaredTerminalEvent
                if self.endpoint.event.is_some_and(|id| {
                    c.events.iter().flatten().any(|e| e.id == id && e.terminal)
                }) => {}
            _ => {
                return Err(contract(
                    "endpoint declaration must name an admitted terminal event or a fixed horizon",
                ));
            }
        }

        c.validate()?;
        let n = c.states.len();
        if self.forward() && c.events.iter().flatten().any(|e| e.terminal) {
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
        if self.sensitivity == DynamicSensitivity::Adjoint {
            self.admit_adjoint(c)?;
        }
        let m = c.outputs.len();
        let np = self.integration_width(c.parameters.len());
        let scheduled = self
            .schedule
            .iter()
            .map(|s| s.parameter)
            .collect::<BTreeSet<_>>();
        if scheduled.len() != self.schedule.len()
            || scheduled.iter().any(|k| *k >= c.parameters.len())
            || self.schedule.iter().any(|s| {
                s.times.is_empty()
                    || s.times
                        .iter()
                        .any(|t| !t.is_finite() || *t <= self.start || *t > self.end)
                    || s.times.windows(2).any(|w| w[0] >= w[1])
            })
        {
            return Err(contract(
                "a scheduled input names one contract parameter once, with increasing change times after the start and up to the end",
            ));
        }
        if !self.start.is_finite()
            || !self.end.is_finite()
            || self.start >= self.end
            || !positive(self.rtol)
            || !positive(self.initial_step)
            || self.atol.len() != n
            || self.atol.iter().any(|v| !positive(*v))
            || p.len() != np
            || p.iter().any(|v| !v.is_finite())
            || self.parameter_scales.len() != c.parameters.len()
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
            || (self.sensitivity != DynamicSensitivity::None && np == 0)
            || (self.forward()
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
                v.checked_mul(if self.forward() {
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
            .and_then(|v| {
                if c.balances.is_empty() {
                    return Some(v);
                }
                let points = self
                    .max_events
                    .checked_mul(3)?
                    .checked_add(self.samples.len())?
                    .checked_add(2)?;
                let width = c
                    .balances
                    .len()
                    .checked_mul(3)?
                    .checked_add(c.quadratures.len())?;
                points.checked_mul(width)?.checked_add(v)
            })
            .and_then(|v| {
                v.checked_add(
                    n.checked_add(c.outputs.len())?
                        .checked_add(c.quadratures.len())?
                        .checked_add(c.parameters.len().checked_mul(2)?)?,
                )
            })
            .ok_or_else(|| contract("dynamic result extent overflow"))?;
        if cells > self.max_cells {
            return Err(ProblemError::memory("dynamic result cell allowance"));
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
    /// The exact second derivatives of `Σᵢ weights[i]·fᵢ` with respect to the state
    /// followed by the parameters, as the lower triangle (row ≥ column) of a square sparse
    /// matrix. Available when [`Contract::derivatives`] is second order; an oracle
    /// without second derivatives refuses.
    fn weighted_hessian(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
    ) -> Result<faer::sparse::SparseColMat<usize, f64>, ProblemError> {
        let _ = (mode, function, time, state, parameters, weights);
        Err(ProblemError::unsupported(
            "the dynamic oracle provides no second derivatives",
        ))
    }
}
/// `H·d` for a symmetric `H` stored as its lower triangle: every strictly lower entry
/// contributes to both of its rows.
#[cfg(feature = "idas")]
pub(crate) fn symmetric_product(
    lower: faer::sparse::SparseColMatRef<'_, usize, f64>,
    direction: &[f64],
) -> Result<Vec<f64>, ProblemError> {
    let n = lower.ncols();
    if lower.nrows() != n || direction.len() != n {
        return Err(ProblemError::internal("weighted Hessian extent"));
    }
    let mut out = vec![0.0; n];
    for col in 0..n {
        for k in lower.col_range(col) {
            let row = lower.row_idx()[k];
            let value = lower.val()[k];
            if row < col {
                return Err(ProblemError::internal(
                    "weighted Hessian entry above the diagonal",
                ));
            }
            out[row] += value * direction[col];
            if row != col {
                out[col] += value * direction[row];
            }
        }
    }
    Ok(out)
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
    /// State-major derivatives with respect to the integration parameters, empty when
    /// not requested.
    pub state_sensitivities: Vec<f64>,
    /// Output-major derivatives with respect to the integration parameters, empty when
    /// not requested.
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
/// Independent original-space conservation facts at an actual trajectory point.
/// Inventories, transfers and defects follow the contract's balance order; integrals
/// follow quadrature order. Result permission belongs to the caller's physical policy.
#[derive(Clone, Debug)]
pub struct ConservationPoint {
    /// Actual physical observation time.
    pub time: f64,
    /// Mode whose inventory definition was evaluated.
    pub mode: usize,
    /// Physical inventory evaluated from the actual consistent state and input segment.
    pub inventories: Vec<f64>,
    /// Cumulative original signed flux quadratures.
    pub integrals: Vec<f64>,
    /// Cumulative evaluated permitted event transfers.
    pub transfers: Vec<f64>,
    /// I(t) - I(t0) - integral(original flux) - sum(permitted transfers).
    pub defects: Vec<f64>,
}
/// Actual completion point, captured separately from the requested output grid.
#[derive(Clone, Debug)]
pub struct TrajectoryEndpoint {
    /// Original state, mode, quadratures and physical outputs at completion.
    pub point: Sample,
    /// Terminal event guard, absent for a fixed-horizon endpoint.
    pub event: Option<SemanticId>,
    /// Live integration column of each contract input, before terminal changes.
    pub input_columns: Vec<usize>,
    /// Live contract input values captured from those columns.
    pub inputs: Vec<f64>,
}
/// Joined report retaining valid completed data even when a later callback fails.
#[derive(Debug)]
pub struct Report {
    /// Native/profile outcome.
    pub termination: Termination,
    /// Last completed physical time; initial time until initialization succeeds.
    pub completed_time: f64,
    /// Actual successful fixed or terminal-event endpoint, even off the output grid.
    pub endpoint: Option<TrajectoryEndpoint>,
    /// Requested initial state and guesses.
    pub requested_initial: Vec<f64>,
    /// Native consistent initial state.
    pub consistent_initial: Vec<f64>,
    /// Completed samples only.
    pub samples: Vec<Sample>,
    /// Actual event transitions only.
    pub events: Vec<EventRecord>,
    /// Original-space closure at the baseline, samples, segment endpoints and settled transitions.
    pub conservation: Vec<ConservationPoint>,
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
    /// Evaluate the declared endpoint and observation coverage without inventing samples.
    pub fn assess_endpoint(&self, profile: &Profile) -> EndpointAssessment {
        use pse_model::generated::enums::EndpointPolicy;
        let satisfied = self.error.is_none()
            && self.endpoint.as_ref().is_some_and(|end| {
                end.point.time == self.completed_time
                    && match profile.endpoint.kind {
                        EndpointPolicy::FixedHorizon => {
                            self.termination == Termination::Completed
                                && end.event.is_none()
                                && end.point.time == profile.end
                        }
                        EndpointPolicy::DeclaredTerminalEvent => {
                            self.termination == Termination::Event
                                && profile.endpoint.event.is_some()
                                && end.event == profile.endpoint.event
                        }
                    }
            });
        let expected = profile
            .samples
            .iter()
            .filter(|t| **t <= self.completed_time)
            .copied()
            .collect::<Vec<_>>();
        let prefix_complete = self.samples.len() == expected.len()
            && self
                .samples
                .iter()
                .zip(expected)
                .all(|(sample, time)| sample.time == time);
        EndpointAssessment {
            satisfied,
            prefix_complete,
            missing_observations: profile
                .samples
                .iter()
                .filter(|t| **t > self.completed_time)
                .copied()
                .collect(),
        }
    }

    /// Known retained trajectory buffers. Opaque statistics/error/progress storage
    /// is covered separately by the runtime's report allowance.
    pub fn numeric_bytes(&self) -> usize {
        size_of::<Self>()
            + (self.requested_initial.capacity() + self.consistent_initial.capacity())
                * size_of::<f64>()
            + self.endpoint.as_ref().map_or(0, |e| {
                size_of::<TrajectoryEndpoint>()
                    + e.input_columns.capacity() * size_of::<usize>()
                    + (e.inputs.capacity()
                        + e.point.integrals.capacity()
                        + e.point.state.capacity()
                        + e.point.outputs.capacity()
                        + e.point.state_sensitivities.capacity()
                        + e.point.output_sensitivities.capacity())
                        * size_of::<f64>()
            })
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
            + self.conservation.capacity() * size_of::<ConservationPoint>()
            + self
                .conservation
                .iter()
                .map(|p| {
                    (p.inventories.capacity()
                        + p.integrals.capacity()
                        + p.transfers.capacity()
                        + p.defects.capacity())
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
            endpoint: None,
            requested_initial: vec![],
            consistent_initial: vec![],
            samples: vec![],
            events: vec![],
            conservation: vec![],
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
            .field("sensitivity", &self.sensitivity)
            .finish_non_exhaustive()
    }
}

/// Every Diffsol-only control, in the serde encoding of the profile (F09): the remote
/// definitions fail to compile when a pinned option type gains a field, and no field
/// list is written by hand.
#[derive(serde::Serialize)]
struct DiffsolIdentity<'a> {
    #[serde(with = "initial_options")]
    initialization: Arc<diffsol::InitialConditionSolverOptions<f64>>,
    #[serde(with = "ode_options")]
    native: Arc<diffsol::OdeSolverOptions<f64>>,
    diffsol: &'a DiffsolSettings,
}
/// Exact Diffsol-only controls included in environment identity and reporting.
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
/// Evaluated pre-transition facts, captured before a reset or an input segment change.
#[cfg(any(feature = "diffsol", feature = "idas"))]
#[derive(Debug)]
pub(crate) struct Transition {
    pub record: usize,
    pub inventories: Vec<f64>,
    pub transfers: Vec<f64>,
}
/// Record physical closure without applying physical acceptance policy. Native algorithms
/// integrate only the declared original flux; this calculation never reads the solved rates.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) fn observe_conservation(
    layout: &Contract,
    report: &mut Report,
    time: f64,
    mode: usize,
    inventories: Vec<f64>,
    integrals: Vec<f64>,
    transfers: Vec<f64>,
) -> Result<(), ProblemError> {
    if layout.balances.is_empty() {
        return Ok(());
    }
    let n = layout.balances.len();
    if inventories.len() != n || transfers.len() != n || integrals.len() != layout.quadratures.len()
    {
        return Err(ProblemError::internal("conservation function dimensions"));
    }
    let baseline = report
        .conservation
        .first()
        .map_or(inventories.as_slice(), |p| p.inventories.as_slice());
    let defects = layout
        .balances
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let q = layout
                .quadratures
                .iter()
                .position(|id| *id == b.flux)
                .ok_or_else(|| ProblemError::internal("conservation flux identity"))?;
            Ok(inventories[i] - baseline[i] - integrals[q] - transfers[i])
        })
        .collect::<Result<Vec<_>, ProblemError>>()?;
    if inventories
        .iter()
        .chain(&integrals)
        .chain(&transfers)
        .chain(&defects)
        .any(|v| !v.is_finite())
    {
        return Err(ProblemError::numerical(
            "nonfinite conservation observation",
        ));
    }
    report.conservation.push(ConservationPoint {
        time,
        mode,
        inventories,
        integrals,
        transfers,
        defects,
    });
    Ok(())
}
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) fn cumulative_transfers(layout: &Contract, report: &Report) -> Vec<f64> {
    report
        .conservation
        .last()
        .map_or_else(|| vec![0.0; layout.balances.len()], |p| p.transfers.clone())
}
/// Both native routes settle the same independently evaluated inventory jump. Scheduled
/// consistency changes transfer zero. Preserve the observed defect even when a reset refuses.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) fn settle_transition(
    layout: &Contract,
    report: &mut Report,
    transition: Transition,
    point: ConservationPoint,
    state: &[f64],
) -> Result<(), ProblemError> {
    let mut transfers = cumulative_transfers(layout, report);
    let invalid = layout.balances.iter().enumerate().any(|(i, b)| {
        (point.inventories[i] - transition.inventories[i] - transition.transfers[i]).abs()
            > b.tolerance
    });
    for (total, jump) in transfers.iter_mut().zip(&transition.transfers) {
        *total += jump;
    }
    observe_conservation(
        layout,
        report,
        point.time,
        point.mode,
        point.inventories,
        point.integrals,
        transfers,
    )?;
    if invalid {
        return Err(contract(
            "conserved inventory jump differs from its permitted event transfer",
        ));
    }
    let record = report
        .events
        .get_mut(transition.record)
        .ok_or_else(|| ProblemError::internal("conservation transition record"))?;
    record.after = Some(state.to_vec());
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

mod initial_options {
    use super::*;
    #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(
        remote = "diffsol::InitialConditionSolverOptions<f64>",
        deny_unknown_fields
    )]
    #[schemars(rename = "DiffsolInitialConditionOptions")]
    pub(super) struct Remote {
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

mod ode_options {
    use super::*;
    #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(remote = "diffsol::OdeSolverOptions<f64>", deny_unknown_fields)]
    #[schemars(rename = "DiffsolOdeSolverOptions")]
    pub(super) struct Remote {
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
const fn no_sensitivity() -> DynamicSensitivity {
    DynamicSensitivity::None
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            method: automatic(),
            trial_failures: terminal(),
            numerics: Default::default(),
            endpoint: EndpointRequirement::default(),
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
            sensitivity: no_sensitivity(),
            adjoint: AdjointSettings::default(),
            parameter_scales: vec![],
            schedule: vec![],
            diffsol: DiffsolSettings::default(),
            idas: IdasSettings::default(),
            initialization: Arc::new(Default::default()),
            native: Arc::new(Default::default()),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "diffsol")]
mod tests;

#[cfg(test)]
mod contextual_tests;
