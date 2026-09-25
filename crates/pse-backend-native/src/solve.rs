// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-neutral execution controls and faithful result envelopes.
use crate::ProblemError;
use pse_ids::{ContentHash, SemanticId};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// Registry-owned tags shared with Arrow and Python.
pub use pse_model::generated::enums::{
    EvidenceUnavailableReason as UnavailableReason, NativeAssurance as Assurance,
    NativeBackend as Backend, NativeCandidateKind as CandidateKind,
    NativeDerivativeCapability as DerivativeCapability, NativeProblemClass as ProblemClass,
    NativeQualification as Qualification, NativeStartPolicy as StartPolicy,
    NativeTermination as Termination, NativeWarmCapability as WarmCapability,
};
/// Adapter behavior stays with the linked implementations, outside shared semantic values.
pub trait BackendCapabilities {
    /// Static inventory; actual model eligibility is a separate admission result.
    fn capabilities(self) -> Capabilities;
    /// Whether this binary links the selected native implementation.
    fn available(self) -> bool;
}
impl BackendCapabilities for Backend {
    /// Explicit representational and execution capabilities of the linked adapters.
    /// Eligibility still requires admission of the actual model and profile.
    fn capabilities(self) -> Capabilities {
        match self {
            Self::Ipopt => Capabilities {
                general_bounds: true,
                sign_bounds: true,
                parallel: false,
                contextual: None,
                classes: &[ProblemClass::SmoothNlp],
                derivatives: DerivativeCapability::ExactHessianOrLimitedMemory,
                warm: WarmCapability::PrimalDual,
                reuse: "same sparse layout and bounds: retained C problem",
                cancellation: "intermediate/evaluation checkpoints",
                diagnostics: "native current iterate, violations, callback counts and timing",
            },
            Self::Pounce => Capabilities {
                general_bounds: true,
                sign_bounds: true,
                parallel: true,
                contextual: None,
                classes: &[ProblemClass::SmoothNlp],
                derivatives: DerivativeCapability::ExactHessianOrLimitedMemory,
                warm: WarmCapability::PrimalDualAndWorkingSet,
                reuse: "native application and compatible starts; iteration factors are library-owned",
                cancellation: "TNLP intermediate/evaluation checkpoints",
                diagnostics: "complete SolveStatistics, phase timing, FERAL inertia/pivots/fill, restoration and crossover",
            },
            Self::Kinsol => Capabilities {
                general_bounds: false,
                sign_bounds: true,
                parallel: false,
                contextual: None,
                classes: &[ProblemClass::SquareRoot, ProblemClass::DeclaredFixedPoint],
                derivatives: DerivativeCapability::JacobianOrProduct,
                warm: WarmCapability::Primal,
                reuse: "same sparse layout: retained SUNDIALS/KLU allocations",
                cancellation: "evaluation checkpoints; native factorization completes before teardown",
                diagnostics: "native nonlinear/linear iterations, setups, failures, norms and callback timing",
            },
            Self::Highs => Capabilities {
                general_bounds: true,
                sign_bounds: true,
                parallel: true,
                contextual: None,
                classes: &[
                    ProblemClass::Linear,
                    ProblemClass::MixedLinear,
                    ProblemClass::ConvexQuadratic,
                ],
                derivatives: DerivativeCapability::Coefficients,
                warm: WarmCapability::PrimalDualAndBasis,
                reuse: "native coefficient/bound updates with compatible layout",
                cancellation: "simplex/IPM/MIP interrupt callbacks; QP native time limit",
                diagnostics: "native information, rays, IIS, ranging and explicit relaxation",
            },
            Self::Clarabel => Capabilities {
                general_bounds: true,
                sign_bounds: true,
                parallel: false,
                contextual: None,
                classes: &[ProblemClass::ContinuousCone],
                derivatives: DerivativeCapability::Coefficients,
                warm: WarmCapability::None,
                reuse: "native data-update eligibility; reusable mode disables preprocessing",
                cancellation: "native iteration termination callback",
                diagnostics: "complete native info/settings, cone slacks/duals and certificates",
            },
            Self::Idas => Capabilities {
                general_bounds: false,
                sign_bounds: false,
                parallel: false,
                contextual: None,
                classes: &[ProblemClass::Ode, ProblemClass::SemiExplicitIndex1],
                derivatives: DerivativeCapability::FirstWithSmoothSensitivities,
                warm: WarmCapability::None,
                reuse: "worker-local IDAS residual state",
                cancellation: "residual callbacks and native step boundaries",
                diagnostics: "native statuses, consistent starts, recoverable residual trials and sensitivities",
            },
            Self::Diffsol => Capabilities {
                general_bounds: false,
                sign_bounds: false,
                parallel: false,
                contextual: None,
                classes: &[ProblemClass::Ode, ProblemClass::SemiExplicitIndex1],
                derivatives: DerivativeCapability::FirstWithSmoothSensitivities,
                warm: WarmCapability::None,
                reuse: "worker-local BDF state",
                cancellation: "cooperative callbacks and step boundaries",
                diagnostics: "native statistics, consistent starts, partial samples and root transitions",
            },
        }
    }
    /// Whether this binary includes the adapter and its native link profile.
    fn available(self) -> bool {
        match self {
            Self::Ipopt => cfg!(feature = "ipopt"),
            Self::Pounce => cfg!(feature = "pounce"),
            Self::Kinsol => cfg!(feature = "kinsol"),
            Self::Highs => cfg!(feature = "highs"),
            Self::Clarabel => true,
            Self::Diffsol => cfg!(feature = "diffsol"),
            Self::Idas => cfg!(feature = "idas"),
        }
    }
}
/// Inspectable adapter contract with explicit operational limitations.
#[derive(Clone, Copy, Debug)]
pub struct Capabilities {
    /// Static bound representation; KINSOL accepts only its sign constraints.
    pub general_bounds: bool,
    /// Whether a compatible strategy can represent sign constraints.
    pub sign_bounds: bool,
    /// Whether the adapter can consume more than one admitted native thread.
    pub parallel: bool,
    /// Selected-model admission is absent from a static inventory. P07 owns routing.
    pub contextual: Option<ContextualCapabilities>,
    /// Representable mathematical classes.
    pub classes: &'static [ProblemClass],
    /// Required derivative representation.
    pub derivatives: DerivativeCapability,
    /// Externally supplied starting-state support.
    pub warm: WarmCapability,
    /// Native allocation/data reuse boundary.
    pub reuse: &'static str,
    /// Actual interrupt checkpoints.
    pub cancellation: &'static str,
    /// Available native diagnostic families.
    pub diagnostics: &'static str,
}
/// Facts established for one selected model/settings pair, never inferred from
/// the static library inventory. The routing boundary produces this record.
#[derive(Clone, Copy, Debug)]
pub struct ContextualCapabilities {
    /// Identity of the exact mathematical representation and selected analysis.
    pub analysis: ContentHash,
    /// Mathematical class proved for that representation.
    pub class: ProblemClass,
    /// Highest derivative order admitted for every selected body and provider.
    pub derivatives: pse_kernels::DerivativeOrder,
    /// Actual native thread count admitted against the runtime budget.
    pub threads: usize,
    /// Bound semantics were checked for the selected native strategy.
    pub bounds_admitted: bool,
    /// Starting payload was checked for the selected layout and backend.
    pub start_admitted: bool,
}
/// Why the caller requests numerical work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolveIntent {
    /// Optimize the authored scalar objective.
    Optimize,
    /// Solve declared square equations.
    Root,
    /// Find feasibility using an explicitly constant NLP objective.
    FeasiblePoint,
    /// Produce a start, without claiming optimization.
    Initialize,
}
/// No implicit fallback is performed for an unavailable selected backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverSelection {
    /// Deterministic mathematical-class routing.
    Auto,
    /// User-selected eligible native implementation.
    Explicit(Backend),
}
/// Native option type; each adapter validates registration, type and protected controls.
#[derive(Clone, Debug, PartialEq)]
pub enum OptionValue {
    /// Native text/enumeration.
    Text(String),
    /// Finite native real.
    Real(f64),
    /// Native integer.
    Integer(i32),
    /// Native Boolean.
    Bool(bool),
}
/// Effective options retain origin, including native defaults when queried.
pub type Options = BTreeMap<String, OptionValue>;
/// Derivative policy does not silently enable finite differences.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HessianMode {
    /// Exact weighted Lagrangian Hessian.
    Exact,
    /// Library-owned quasi-Newton approximation.
    LimitedMemory,
}
/// Compatible native state retention requirement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReusePolicy {
    /// Always construct a fresh native model.
    Fresh,
    /// Rebuild explicitly when data updates are ineligible.
    AllowRebuild,
    /// Fail rather than rebuilding incompatible native state.
    RequireReuse,
}
/// Independently resolved native stopping controls in normalized coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct Accuracy {
    /// Conservative scalar projection of comparable normalized feasibility budgets.
    pub feasibility: f64,
    /// Normalized dual stationarity budget.
    pub stationarity: f64,
    /// Normalized complementarity budget.
    pub complementarity: f64,
    /// Original integer-lattice violation budget.
    pub integrality: f64,
    /// Absolute continuous primal-dual gap budget.
    pub gap_absolute: f64,
    /// Relative continuous primal-dual gap budget.
    pub gap_relative: f64,
    /// Absolute mixed-integer objective gap.
    pub mip_absolute_gap: f64,
    /// Relative mixed-integer objective gap.
    pub mip_relative_gap: f64,
    /// Explicit relaxed KKT acceptance, absent by default.
    pub acceptable: Option<pse_model::numerics::KktTolerances>,
    /// Native algorithmic scaling, separate from the model coordinate transform.
    pub native_scaling: bool,
}
impl Default for Accuracy {
    fn default() -> Self {
        Self {
            feasibility: 1e-8,
            stationarity: 1e-8,
            complementarity: 1e-8,
            integrality: 1e-8,
            gap_absolute: 1e-8,
            gap_relative: 1e-8,
            mip_absolute_gap: 1e-6,
            mip_relative_gap: 1e-4,
            acceptable: None,
            native_scaling: true,
        }
    }
}
impl Accuracy {
    /// Shared Ipopt-compatible stopping and original-bound contract for both NLP adapters.
    pub fn nlp_options(&self) -> BTreeMap<String, OptionValue> {
        BTreeMap::from([
            ("tol".into(), OptionValue::Real(self.nlp_tolerance())),
            (
                "constr_viol_tol".into(),
                OptionValue::Real(self.feasibility),
            ),
            ("dual_inf_tol".into(), OptionValue::Real(self.stationarity)),
            (
                "compl_inf_tol".into(),
                OptionValue::Real(self.complementarity),
            ),
            ("bound_relax_factor".into(), OptionValue::Real(0.0)),
            ("honor_original_bounds".into(), OptionValue::Bool(true)),
            (
                "acceptable_iter".into(),
                OptionValue::Integer(if self.acceptable.is_some() { 15 } else { 0 }),
            ),
            (
                "acceptable_tol".into(),
                OptionValue::Real(self.acceptable.map_or(self.nlp_tolerance(), |k| {
                    k.stationarity.min(k.complementarity)
                })),
            ),
            (
                "acceptable_constr_viol_tol".into(),
                OptionValue::Real(self.feasibility),
            ),
            (
                "acceptable_dual_inf_tol".into(),
                OptionValue::Real(
                    self.acceptable
                        .map_or(self.stationarity, |k| k.stationarity),
                ),
            ),
            (
                "acceptable_compl_inf_tol".into(),
                OptionValue::Real(
                    self.acceptable
                        .map_or(self.complementarity, |k| k.complementarity),
                ),
            ),
        ])
    }
    /// Complete native numerical contract identity, including independent stopping budgets.
    pub fn key(&self) -> ContentHash {
        let mut h = pse_ids::FramedHasher::new("pse.native.accuracy.v1");
        for v in [
            self.feasibility,
            self.stationarity,
            self.complementarity,
            self.integrality,
            self.gap_absolute,
            self.gap_relative,
            self.mip_absolute_gap,
            self.mip_relative_gap,
        ] {
            h.u64(v.to_bits());
        }
        h.bool(self.native_scaling).bool(self.acceptable.is_some());
        if let Some(k) = self.acceptable {
            h.u64(k.stationarity.to_bits())
                .u64(k.complementarity.to_bits());
        }
        h.finish_hash()
    }
    /// Derive semantic native controls; physical arrays remain the final acceptance authority.
    pub fn resolve(
        policy: &pse_model::numerics::NumericalPolicy,
        tolerance: &crate::quality::Tolerances,
        normalization: &pse_math::normalization::Normalization,
    ) -> Result<Self, ProblemError> {
        policy
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let tolerance = tolerance.normalized(normalization)?;
        let feasibility = tolerance
            .variables
            .iter()
            .chain(&tolerance.rows)
            .copied()
            .reduce(f64::min)
            .unwrap_or(1e-8);
        Ok(Self {
            feasibility,
            stationarity: policy.kkt.stationarity,
            complementarity: policy.kkt.complementarity,
            integrality: policy.integrality,
            gap_absolute: policy.gap_absolute,
            gap_relative: policy.gap_relative,
            mip_absolute_gap: policy.mip_absolute_gap,
            mip_relative_gap: policy.mip_relative_gap,
            acceptable: policy.acceptable,
            native_scaling: policy.native_scaling,
        })
    }
    /// Native dimensionless controls must remain finite and strictly positive where required.
    fn valid(&self) -> bool {
        [
            self.feasibility,
            self.stationarity,
            self.complementarity,
            self.integrality,
            self.gap_absolute,
            self.gap_relative,
        ]
        .into_iter()
        .all(|v| v.is_finite() && v > 0.0)
            && [self.mip_absolute_gap, self.mip_relative_gap]
                .into_iter()
                .all(|v| v.is_finite() && v >= 0.0)
            && self.acceptable.is_none_or(|k| {
                k.stationarity.is_finite()
                    && k.stationarity >= self.stationarity
                    && k.complementarity.is_finite()
                    && k.complementarity >= self.complementarity
            })
    }
    /// Combined native NLP error budget; separate unscaled criteria remain explicit.
    pub fn nlp_tolerance(&self) -> f64 {
        self.feasibility
            .min(self.stationarity)
            .min(self.complementarity)
    }
}
/// Finite shared attempt controls; solver-specific settings remain native typed values.
#[derive(Clone, Debug)]
pub struct Controls {
    /// Positive wall-clock allowance, including callbacks.
    pub time_limit: Duration,
    /// Positive native iteration limit.
    pub iterations: u32,
    /// Independently resolved native accuracy controls.
    pub accuracy: Accuracy,
    /// Explicit admitted native thread count.
    pub threads: usize,
    /// Bounded retained progress and failure events.
    pub history: usize,
    /// Hessian representation for compatible NLP methods.
    pub hessian: HessianMode,
    /// Native allocation/data reuse policy.
    pub reuse: ReusePolicy,
    /// Numerical start policy, independent of native allocation reuse.
    pub start: StartPolicy,
    /// Additional native options, admitted by the selected adapter.
    pub options: Options,
}
impl Default for Controls {
    fn default() -> Self {
        Self {
            time_limit: Duration::from_secs(300),
            iterations: 3000,
            accuracy: Accuracy::default(),
            threads: 1,
            history: 256,
            hessian: HessianMode::Exact,
            reuse: ReusePolicy::Fresh,
            start: StartPolicy::NoPriorStart,
            options: Options::new(),
        }
    }
}
impl Controls {
    /// Conservative retained reporting allowance, separate from worker/native scratch.
    /// Native option readback, explicit strings and bounded event copies are included.
    pub fn report_allowance(&self) -> Result<usize, ProblemError> {
        let options = self.options.iter().try_fold(0usize, |n, (k, v)| {
            let value = match v {
                OptionValue::Text(v) => v.len(),
                _ => 16,
            };
            n.checked_add(k.len())
                .and_then(|n| n.checked_add(value))
                .and_then(|n| n.checked_add(256))
        });
        options
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| {
                self.history
                    .checked_mul(8192)
                    .and_then(|h| n.checked_add(h))
            })
            .and_then(|n| n.checked_add(4 << 20))
            .ok_or_else(|| ProblemError::Contract("report allowance overflow".into()))
    }
    /// Refuse unlimited/invalid controls before allocating native state.
    pub fn validate(&self) -> Result<(), ProblemError> {
        if self.time_limit.is_zero()
            || self.iterations == 0
            || self.iterations > i32::MAX as u32
            || !self.accuracy.valid()
            || self.threads == 0
            || self.threads > i32::MAX as usize
            || self.history > 1_000_000
            || self.options.len() > 4096
            || self.options.iter().any(|(k, v)| {
                k.is_empty()
                    || k.len() > 256
                    || k.contains('\0')
                    || match v {
                        OptionValue::Real(v) => !v.is_finite(),
                        OptionValue::Text(v) => v.len() > 4096 || v.contains('\0'),
                        _ => false,
                    }
            })
        {
            return Err(ProblemError::Contract(
                "invalid finite solve controls".into(),
            ));
        }
        Ok(())
    }
}
/// Raw native termination and the adapter's conservative interpretation.
#[derive(Clone, Debug)]
pub struct NativeTermination {
    /// Unmodified native numeric status.
    pub code: i64,
    /// Native symbolic status name.
    pub name: String,
    /// Native diagnostic, when available.
    pub message: Option<String>,
    /// Cross-backend category.
    pub category: Termination,
    /// Native assurance, subsequently constrained by original-model validation.
    pub assurance: Assurance,
}
/// Lossless native data: integral counters are not rounded into floating-point numbers.
#[derive(Clone, Debug, PartialEq)]
pub enum Metric {
    /// Count/status identifier.
    Integer(i64),
    /// Native real measurement with its documented unit.
    Real(f64),
    /// Native diagnostic/enum.
    Text(String),
    /// Explicit native Boolean.
    Bool(bool),
    /// An absent observation, with a stable reason instead of a text sentinel.
    Unavailable(UnavailableReason),
}
/// One owned, bounded event. No pointers or borrowed native buffers escape.
#[derive(Clone, Debug)]
pub struct Event {
    /// Phase or callback name.
    pub phase: String,
    /// Time since admitted execution began.
    pub elapsed: Duration,
    /// Native event values with backend-specific keys.
    pub values: BTreeMap<String, Metric>,
}
/// A bounded event stream shared with an observing handle.
#[derive(Debug)]
pub struct Progress {
    limit: usize,
    events: Mutex<(Vec<Event>, u64)>,
}
impl Metric {
    /// Stable type tag; raw nonfinite native values are observations, not missing sentinels.
    pub const fn kind(&self) -> pse_model::generated::enums::NativeMetricKind {
        use pse_model::generated::enums::NativeMetricKind;
        match self {
            Self::Integer(_) => NativeMetricKind::Integer,
            Self::Real(_) => NativeMetricKind::Real,
            Self::Text(_) => NativeMetricKind::Text,
            Self::Bool(_) => NativeMetricKind::Boolean,
            Self::Unavailable(_) => NativeMetricKind::Unavailable,
        }
    }
}

impl Progress {
    /// Allocate only up to the admitted bound; later events increment a dropped count.
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            events: Mutex::new((Vec::new(), 0)),
        }
    }
    /// Copy an owned event into the bounded stream.
    pub fn push(&self, event: Event) {
        if let Ok(mut s) = self.events.lock() {
            let bytes =
                event
                    .values
                    .iter()
                    .fold(event.phase.len().saturating_add(64), |n, (k, v)| {
                        n.saturating_add(k.len())
                            .saturating_add(128)
                            .saturating_add(match v {
                                Metric::Text(v) => v.len(),
                                _ => 0,
                            })
                    });
            if s.0.len() < self.limit && bytes <= 4096 {
                s.0.push(event);
            } else {
                s.1 = s.1.saturating_add(1);
            }
        }
    }
    /// Snapshot currently retained events and their dropped-event count.
    pub fn snapshot(&self) -> (Vec<Event>, u64) {
        self.events.lock().map(|s| s.clone()).unwrap_or_default()
    }
}
/// Cancellation and deadline checkpoints are shared by every callback adapter.
#[derive(Clone, Debug)]
pub struct Execution {
    /// Cooperative cancellation; native teardown still must complete.
    pub cancel: Arc<AtomicBool>,
    /// Attempt start, set after admission.
    pub started: Instant,
    /// Positive admitted limit.
    pub time_limit: Duration,
    /// Bounded retained progress.
    pub progress: Arc<Progress>,
}
impl Execution {
    /// Construct at the admitted worker boundary.
    pub fn new(cancel: Arc<AtomicBool>, controls: &Controls) -> Self {
        Self {
            cancel,
            started: Instant::now(),
            time_limit: controls.time_limit,
            progress: Arc::new(Progress::new(controls.history)),
        }
    }
    /// Stop reason; cancellation and deadline remain distinguishable.
    pub fn stopped(&self) -> Option<Termination> {
        if self.cancel.load(Ordering::Acquire) {
            Some(Termination::Cancelled)
        } else if self.started.elapsed() >= self.time_limit {
            Some(Termination::TimeLimit)
        } else {
            None
        }
    }
}
/// Primal and available native dual data, in declared source order.
#[derive(Clone, Debug)]
pub struct Candidate {
    /// What the native API actually supplied, independent of qualification.
    pub kind: CandidateKind,
    /// Original independent variable values.
    pub primal: Vec<f64>,
    /// Authored objective, including sense-independent constant.
    pub objective: Option<f64>,
    /// Native row multipliers; sign convention is recorded by the adapter.
    pub row_dual: Option<Vec<f64>>,
    /// Separate lower/upper bound duals only where supplied by the native API.
    pub bound_dual: Option<(Vec<f64>, Vec<f64>)>,
    /// Native reduced costs, not a fabricated split bound dual.
    pub reduced_costs: Option<Vec<f64>>,
    /// Native slack vector, when meaningful.
    pub slacks: Option<Vec<f64>>,
}
/// Reuse compatibility separates semantic layout from numeric data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Compatibility {
    /// Includes source IDs, domains, physical maps, scaling, sparse structure and profile.
    pub layout: ContentHash,
    /// Parameters, fixed values, coefficients and bounds for this attempt.
    pub data: ContentHash,
    /// Native backend/profile identity.
    pub backend: Backend,
}
/// Native basis values retain their original integer codes.
#[derive(Clone, Debug)]
pub struct Basis {
    /// Native column basis statuses.
    pub columns: Vec<i32>,
    /// Native row basis statuses.
    pub rows: Vec<i32>,
}
/// Starts are typed by their native mathematical meaning.
#[derive(Clone, Debug)]
pub enum WarmPayload {
    /// POUNCE active-set SQP iterate and working set in its native vocabulary.
    #[cfg(feature = "pounce")]
    PounceSqp(pounce_rs::pounce_algorithm::sqp::SqpIterates),
    /// Primal start, optionally with NLP lower/upper/row multipliers.
    Nlp {
        /// Primal in source order.
        primal: Vec<f64>,
        /// Lower/upper bound multipliers.
        bounds: Option<(Vec<f64>, Vec<f64>)>,
        /// Constraint multipliers.
        rows: Option<Vec<f64>>,
    },
    /// Root-system initial values; no fictitious duals.
    Root(Vec<f64>),
    /// `HiGHS` primal/dual start and optional simplex basis.
    Highs {
        /// Complete or absent primal.
        primal: Option<Vec<f64>>,
        /// Native column and row dual vectors.
        dual: Option<(Vec<f64>, Vec<f64>)>,
        /// Native simplex basis.
        basis: Option<Basis>,
    },
}
/// Portable owned seed; mutable native model state never crosses a worker boundary.
#[derive(Clone, Debug)]
pub struct WarmStart {
    /// Execution that produced this seed; absent for an explicitly authored seed.
    pub origin: Option<SeedOrigin>,
    /// Compatibility evidence.
    pub compatibility: Compatibility,
    /// Native-class seed.
    pub payload: WarmPayload,
}
/// Portable provenance of an output seed, distinct from its coordinate compatibility.
#[derive(Clone, Debug, serde::Serialize)]
pub struct SeedOrigin {
    /// Public run identity when one exists.
    pub run: Option<SemanticId>,
    /// Zero-based original attempt.
    pub attempt: usize,
}
impl WarmStart {
    /// Check source-coordinate shape and finite payloads before native construction.
    pub fn validate_shape(&self, variables: usize, rows: usize) -> Result<(), ProblemError> {
        let finite = |v: &[f64], n| v.len() == n && v.iter().all(|v| v.is_finite());
        let valid = match &self.payload {
            WarmPayload::Root(x) => {
                self.compatibility.backend == Backend::Kinsol && finite(x, variables)
            }
            WarmPayload::Nlp {
                primal,
                bounds,
                rows: dual,
            } => {
                matches!(self.compatibility.backend, Backend::Ipopt | Backend::Pounce)
                    && finite(primal, variables)
                    && bounds
                        .as_ref()
                        .is_none_or(|(l, u)| finite(l, variables) && finite(u, variables))
                    && dual.as_ref().is_none_or(|d| finite(d, rows))
            }
            WarmPayload::Highs {
                primal,
                dual,
                basis,
            } => {
                self.compatibility.backend == Backend::Highs
                    && (primal.is_some() || dual.is_some() || basis.is_some())
                    && primal.as_ref().is_none_or(|p| finite(p, variables))
                    && dual
                        .as_ref()
                        .is_none_or(|(c, r)| finite(c, variables) && finite(r, rows))
                    && basis.as_ref().is_none_or(|b| {
                        b.columns.len() == variables
                            && b.rows.len() == rows
                            && b.columns.iter().chain(&b.rows).all(|s| (0..=4).contains(s))
                    })
            }
            #[cfg(feature = "pounce")]
            WarmPayload::PounceSqp(s) => {
                self.compatibility.backend == Backend::Pounce
                    && finite(&s.x, variables)
                    && finite(&s.lambda_g, rows)
                    && s.lambda_x.iter().all(|v| v.is_finite())
            }
        };
        if valid {
            Ok(())
        } else {
            Err(ProblemError::Contract(
                "invalid source-coordinate seed payload for selected backend".into(),
            ))
        }
    }
    /// Owned semantic seed snapshot for result provenance; this is not native consumption evidence.
    pub fn snapshot(&self) -> serde_json::Value {
        let payload = match &self.payload {
            WarmPayload::Root(x) => serde_json::json!({"kind":"root","primal":x}),
            WarmPayload::Nlp {
                primal,
                bounds,
                rows,
            } => {
                serde_json::json!({"kind":"nlp","primal":primal,"bound_duals":bounds,"row_duals":rows})
            }
            WarmPayload::Highs {
                primal,
                dual,
                basis,
            } => {
                serde_json::json!({"kind":"highs","primal":primal,"dual":dual,"basis":basis.as_ref().map(|b|serde_json::json!({"columns":b.columns,"rows":b.rows}))})
            }
            #[cfg(feature = "pounce")]
            WarmPayload::PounceSqp(s) => {
                serde_json::json!({"kind":"pounce_sqp","primal":s.x,"row_duals":s.lambda_g,"packed_bound_duals":s.lambda_x,"working_set":s.working.as_ref().map(|w|serde_json::json!({"bounds":w.bounds.iter().map(|v|format!("{v:?}")).collect::<Vec<_>>(),"constraints":w.constraints.iter().map(|v|format!("{v:?}")).collect::<Vec<_>>()}))})
            }
        };
        serde_json::json!({"origin":self.origin,"layout":self.compatibility.layout.to_hex(),"data":self.compatibility.data.to_hex(),"backend":self.compatibility.backend.as_str(),"payload":payload})
    }
    /// Numeric data changes may reuse a seed; layout and backend must match exactly.
    pub fn validate(&self, target: &Compatibility) -> Result<(), ProblemError> {
        if self.compatibility.layout != target.layout
            || self.compatibility.backend != target.backend
        {
            return Err(ProblemError::Contract(
                "incompatible warm-start layout/backend".into(),
            ));
        }
        Ok(())
    }
}
/// Exact owned submitted seed with source-coordinate transformation provenance.
#[derive(Clone, Debug)]
pub struct StartReceipt {
    /// Previous sequence attempt when that policy selected the seed.
    pub previous_attempt: Option<usize>,
    /// Source compatibility and submitted payload in original coordinates.
    pub seed: Option<WarmStart>,
    /// Explicit partial MIP seed in original coordinates, if selected.
    pub sparse_seed: Option<BTreeMap<SemanticId, f64>>,
    /// Transformations subsequently applied by the native transport.
    pub transformations: Vec<String>,
    /// API submission is observable; native internal consumption may remain unavailable.
    pub submitted: bool,
}
/// One native attempt, including unsuccessful attempts with no usable candidate.
#[derive(Clone, Debug)]
pub struct Certificate {
    /// Native certificate type and accuracy qualifier.
    pub kind: String,
    /// Native primal recession direction, if supplied.
    pub primal: Option<Vec<f64>>,
    /// Native dual certificate in original native row order, if supplied.
    pub dual: Option<Vec<f64>>,
}
/// One native attempt, including unsuccessful attempts with no usable candidate.
#[derive(Clone, Debug)]
pub struct SolveReport {
    /// Fresh original-model values and qualified dual diagnostics.
    pub observation: Option<crate::quality::Observation>,
    /// Effective library transformations, including unavailable passes.
    pub preprocessing: Option<crate::presolve::Report>,
    // Keeps a runtime-owned result reservation alive when an envelope is extracted.
    owner: Option<Arc<dyn pse_math::AllocationOwner>>,
    /// Explicitly requested HiGHS diagnostics, separate from the original candidate.
    #[cfg(feature = "highs")]
    pub highs_diagnostics: Option<Box<crate::highs::diagnostics::Report>>,
    /// Complete native POUNCE statistics, with its optional trajectory bounded by policy.
    #[cfg(feature = "pounce")]
    pub pounce_statistics: Option<Box<pounce_rs::SolveStatistics>>,
    /// Native infeasibility/unboundedness certificates, never exposed as primal solutions.
    pub certificate: Option<Certificate>,
    /// Selected native implementation.
    pub backend: Backend,
    /// Declared source variable order.
    pub variables: Vec<SemanticId>,
    /// Declared source row order.
    pub rows: Vec<SemanticId>,
    /// Actual native result, distinct from original quality.
    pub termination: NativeTermination,
    /// Available values only; an iterate does not imply feasibility.
    pub candidate: Option<Candidate>,
    /// Independent original-space checks, or an explicit validation error.
    pub quality: Option<crate::quality::Quality>,
    /// Reason independent validation could not be completed.
    pub validation_error: Option<String>,
    /// Native and adapter metrics; absent means unavailable, never implicitly zero.
    pub metrics: BTreeMap<String, Metric>,
    /// Effective explicit native options and semantic controls.
    pub options: Options,
    /// Native defaults when the library exposes a complete readback API.
    pub native_defaults: Options,
    /// Native build and mathematical convention provenance.
    pub provenance: BTreeMap<String, String>,
    /// Bounded events copied after the native object finishes.
    pub events: Vec<Event>,
    /// Number of events omitted by the admitted history bound.
    pub dropped_events: u64,
    /// Compatible owned seed for a later attempt, when supplied.
    pub warm_start: Option<WarmStart>,
    /// Input seed actually submitted, distinct from the output seed.
    pub start_receipt: Option<StartReceipt>,
    /// Original-space numerical qualification, never inferred from a native stop alone.
    pub qualification: Qualification,
}
impl SolveReport {
    /// Attach the outer runtime's retained-result admission to this owned envelope.
    pub fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.owner = Some(owner);
        self
    }
    /// Start an envelope without inventing unavailable metrics or candidates.
    pub fn new(
        backend: Backend,
        contract: &crate::OracleContract,
        termination: NativeTermination,
        execution: &Execution,
    ) -> Self {
        let (events, dropped_events) = execution.progress.snapshot();
        Self {
            owner: None,
            observation: None,
            preprocessing: None,
            certificate: None,
            #[cfg(feature = "highs")]
            highs_diagnostics: None,
            #[cfg(feature = "pounce")]
            pounce_statistics: None,
            backend,
            variables: contract.variables.iter().map(|v| v.id).collect(),
            rows: contract.rows.clone(),
            termination,
            candidate: None,
            quality: None,
            validation_error: None,
            metrics: BTreeMap::new(),
            options: Options::new(),
            native_defaults: Options::new(),
            provenance: BTreeMap::new(),
            events,
            dropped_events,
            warm_start: None,
            start_receipt: None,
            qualification: Qualification::Unqualified,
        }
    }
}
/// Guard reserved semantic options before asking the native library to validate others.
#[cfg(any(feature = "ipopt", feature = "pounce", feature = "highs"))]
pub(crate) fn reject_reserved(options: &Options, reserved: &[&str]) -> Result<(), ProblemError> {
    if let Some(key) = options
        .keys()
        .find(|k| reserved.contains(&k.rsplit('.').next().unwrap_or(k.as_str())))
    {
        return Err(ProblemError::Contract(format!(
            "native option {key} conflicts with typed controls"
        )));
    }
    Ok(())
}

/// Mechanical projection of a library's serialization contract into typed metric leaves.
/// Null is recorded explicitly, never replaced by zero. This is observational data.
#[cfg(feature = "pounce")]
pub(crate) fn insert_native_metrics(
    out: &mut BTreeMap<String, Metric>,
    prefix: &str,
    value: serde_json::Value,
) {
    use serde_json::Value;
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                insert_native_metrics(out, &format!("{prefix}.{key}"), value);
            }
        }
        Value::Array(values) => {
            for (i, value) in values.into_iter().enumerate() {
                insert_native_metrics(out, &format!("{prefix}.{i}"), value);
            }
        }
        Value::Null => {
            out.insert(
                prefix.into(),
                Metric::Unavailable(UnavailableReason::Unknown),
            );
        }
        Value::Bool(v) => {
            out.insert(prefix.into(), Metric::Bool(v));
        }
        Value::String(v) => {
            out.insert(prefix.into(), Metric::Text(v));
        }
        Value::Number(v) => {
            let metric = if let Some(v) = v.as_i64() {
                Metric::Integer(v)
            } else if let Some(v) = v.as_u64() {
                Metric::Text(v.to_string())
            } else if let Some(v) = v.as_f64() {
                Metric::Real(v)
            } else {
                Metric::Text(v.to_string())
            };
            out.insert(prefix.into(), metric);
        }
    }
}

#[cfg(test)]
mod numerical_tests {
    use super::*;
    #[test]
    fn numerical_options_keep_feasibility_kkt_and_acceptable_independent() {
        let mut accuracy = Accuracy {
            feasibility: 1e-7,
            stationarity: 2e-8,
            complementarity: 3e-9,
            ..Default::default()
        };
        let options = accuracy.nlp_options();
        assert!(matches!(options["constr_viol_tol"],OptionValue::Real(v) if v==1e-7));
        assert!(matches!(options["dual_inf_tol"],OptionValue::Real(v) if v==2e-8));
        assert!(matches!(options["compl_inf_tol"],OptionValue::Real(v) if v==3e-9));
        assert!(matches!(
            options["acceptable_iter"],
            OptionValue::Integer(0)
        ));
        assert!(matches!(
            options["bound_relax_factor"],
            OptionValue::Real(0.0)
        ));
        assert!(matches!(
            options["honor_original_bounds"],
            OptionValue::Bool(true)
        ));
        let key = accuracy.key();
        accuracy.acceptable = Some(pse_model::numerics::KktTolerances {
            stationarity: 1e-5,
            complementarity: 1e-6,
        });
        assert_ne!(key, accuracy.key());
        assert!(matches!(
            accuracy.nlp_options()["acceptable_iter"],
            OptionValue::Integer(15)
        ));
        assert!(
            matches!(accuracy.nlp_options()["acceptable_constr_viol_tol"],OptionValue::Real(v) if v==1e-7)
        );
        let scales = pse_math::normalization::Normalization {
            variables: vec![1e6, 1e-3],
            rows: vec![1e9, 1.0],
            objective: 1.0,
        };
        let t = crate::quality::Tolerances {
            variables: vec![1e-1, 1e-10],
            rows: vec![1e2, 1e-7],
            integrality: 1e-8,
        };
        let resolved = Accuracy::resolve(&Default::default(), &t, &scales).unwrap();
        assert!((resolved.feasibility - 1e-7).abs() < 1e-20);
    }
}
