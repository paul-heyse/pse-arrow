// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-neutral execution controls and faithful result envelopes.
use crate::ProblemError;
use pse_ids::{ContentHash, SemanticId};
use pse_model::{
    scalar,
    scalars::{Fraction, Tolerance},
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

/// Registry-owned tags shared with Arrow and Python.
pub use pse_model::generated::enums::{
    EvidenceUnavailableReason as UnavailableReason, NativeAssurance as Assurance,
    NativeBackend as Backend, NativeCandidateKind as CandidateKind,
    NativeDerivativeCapability as DerivativeCapability, NativeProblemClass as ProblemClass,
    NativeQualification as Qualification, NativeSolveIntent as SolveIntent,
    NativeStartPolicy as StartPolicy, NativeTermination as Termination,
    NativeWarmCapability as WarmCapability,
};
/// Registry-owned vocabularies of the solve controls and the settings documents; the serde
/// spelling of each member is its one name across the Python boundary (ADR-0113, ADR-0115).
/// `HessianMode` selects the Hessian representation of compatible NLP methods, and never
/// silently enables finite differences. `Preconditioner` preconditions a native Krylov
/// linear solve (KINSOL `KINSetPreconditioner`, IDAS `IDASetPreconditioner`) from the
/// compiled analytic Jacobian. `ReusePolicy` states the native state retention
/// requirement. `IpoptLinearSolver` is a symmetric-indefinite solver linked into the
/// image's one Ipopt (ADR-0108), selected by the Ipopt settings and by SCIP's nested Ipopt
/// (ADR-0105 §5); its spelling is Ipopt's native `linear_solver` value.
pub use pse_model::generated::enums::{HessianMode, IpoptLinearSolver, Preconditioner, ReusePolicy};
/// No implicit fallback is performed for an unavailable selected backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverSelection {
    /// Deterministic mathematical-class routing.
    Auto,
    /// User-selected eligible native implementation.
    Explicit(Backend),
}
/// Native option type; each adapter validates registration, type and protected controls.
/// A document states the value itself: a JSON Boolean, integer, other number or string
/// decodes as the Boolean, integer, real or text option of that value.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum OptionValue {
    /// Native Boolean.
    Bool(bool),
    /// Native integer.
    Integer(i32),
    /// Finite native real.
    Real(f64),
    /// Native text/enumeration.
    Text(String),
}
/// Effective options retain origin, including native defaults when queried.
pub type Options = BTreeMap<String, OptionValue>;
/// Native stopping budgets in normalized coordinates, resolved from the numerical policy
/// and the solved function's acceptance budgets (F20). They are never user input: user
/// [`Controls`] carry no accuracy, and every attempt receives the value its preparation
/// resolved. Nested library solves resolve their own from their budgets.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ResolvedAccuracy {
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
impl ResolvedAccuracy {
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
    /// Complete identity of the resolved budgets, derived from serde (F09).
    ///
    /// # Errors
    /// The identity serializer refused a value.
    pub fn key(&self) -> Result<ContentHash, ProblemError> {
        crate::identity::of(pse_ids::Frame::NativeAccuracyV3, self)
    }
    /// Derive semantic native controls; physical arrays remain the final acceptance authority.
    ///
    /// # Errors
    /// An invalid policy or budget, or budgets that resolve to invalid native controls.
    pub fn resolve(
        policy: &pse_model::numerics::NumericalPolicy,
        tolerance: &crate::quality::Tolerances,
        normalization: &pse_math::normalization::Normalization,
    ) -> Result<Self, ProblemError> {
        let tolerance = tolerance.normalized(normalization)?;
        let feasibility = tolerance
            .variables
            .iter()
            .chain(&tolerance.rows)
            .copied()
            .reduce(f64::min)
            .unwrap_or(1e-8);
        Self::from_policy(policy, feasibility)
    }
    /// Resolve the policy's budgets against an already normalized feasibility budget, as
    /// nested library solves do from their own budgets.
    ///
    /// # Errors
    /// An invalid policy, or budgets that resolve to invalid native controls.
    pub fn from_policy(
        policy: &pse_model::numerics::NumericalPolicy,
        feasibility: f64,
    ) -> Result<Self, ProblemError> {
        policy
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let resolved = Self {
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
        };
        resolved.validate()?;
        Ok(resolved)
    }
    /// The default policy's budgets at a normalized feasibility budget of 1e-8.
    #[cfg(test)]
    pub(crate) fn nominal() -> Self {
        Self::from_policy(&Default::default(), 1e-8).unwrap_or_else(|e| panic!("{e}"))
    }
    /// Native dimensionless controls must remain finite and strictly positive where required.
    ///
    /// # Errors
    /// A nonfinite or nonpositive budget, or acceptable budgets tighter than the KKT ones.
    pub fn validate(&self) -> Result<(), ProblemError> {
        if self.valid() {
            Ok(())
        } else {
            Err(ProblemError::Contract(
                "invalid resolved native accuracy".into(),
            ))
        }
    }
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
/// Finite shared attempt controls chosen by the caller; solver-specific settings remain
/// native typed values. Accuracy is not among them: it is resolved from the numerical
/// policy ([`ResolvedAccuracy`]). A document states the time limit in seconds; absent
/// fields take these defaults and unknown fields are refused.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "SolveControls")]
pub struct Controls {
    /// Positive wall-clock allowance, including callbacks, in seconds.
    #[serde(with = "seconds")]
    #[schemars(with = "Tolerance")]
    pub time_limit: Duration,
    /// Positive native iteration limit.
    pub iterations: u32,
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
            threads: 1,
            history: 256,
            hessian: HessianMode::Exact,
            reuse: ReusePolicy::Fresh,
            start: StartPolicy::NoPriorStart,
            options: Options::new(),
        }
    }
}
/// A duration as its seconds: the wall-clock allowances of a document. Decoding refuses a
/// negative, non-finite or unrepresentable number of seconds.
pub(crate) mod seconds {
    use serde::{Deserialize, de, ser};
    use std::time::Duration;

    pub(crate) fn serialize<S: ser::Serializer>(
        value: &Duration,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(value.as_secs_f64())
    }
    pub(crate) fn deserialize<'de, D: de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Duration, D::Error> {
        let seconds = f64::deserialize(deserializer)?;
        Duration::try_from_secs_f64(seconds)
            .map_err(|e| de::Error::custom(format!("{seconds} seconds: {e}")))
    }
}
impl Controls {
    /// Complete identity of these controls, derived from serde (F09).
    ///
    /// # Errors
    /// The identity serializer refused a value.
    pub fn identity(&self) -> Result<ContentHash, ProblemError> {
        crate::identity::of(pse_ids::Frame::NativeControlsV2, self)
    }
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
            .ok_or_else(|| ProblemError::memory("report allowance overflow"))
    }
    /// Refuse unlimited/invalid controls before allocating native state.
    pub fn validate(&self) -> Result<(), ProblemError> {
        if self.time_limit.is_zero()
            || self.iterations == 0
            || self.iterations > i32::MAX as u32
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
    /// A new incumbent of a branch-and-bound search, typed; its bounds are not repeated
    /// among `values`.
    pub incumbent: Option<IncumbentEvent>,
}
/// A new incumbent of a branch-and-bound search, reported while it runs (Plan 22 G8): the
/// objective in original units under the post-solve convention (authored sense, the
/// objective constant and the export offset applied), the search's bounds at that moment,
/// and, when the adapter's capture throttle admits it, the solution itself in original
/// source coordinates, the coordinates a primal warm start of the same layout takes.
#[derive(Clone, Debug, PartialEq)]
pub struct IncumbentEvent {
    /// The incumbent's objective value; always finite.
    pub objective: f64,
    /// The global dual bound, in the objective's units; absent while none is finite.
    pub dual_bound: Option<f64>,
    /// The native relative gap; absent while it is not finite.
    pub gap: Option<f64>,
    /// Branch-and-bound nodes explored by then.
    pub nodes: i64,
    /// Native running time when it was reported, in seconds.
    pub seconds: f64,
    /// The solution in source order; absent when the capture throttle deferred it.
    pub primal: Option<Vec<f64>>,
}
impl IncumbentEvent {
    /// The shortest interval between two captured solutions of one attempt. A solution
    /// found sooner is reported without its primal, and the adapter captures the incumbent
    /// once the interval has passed (or when the search ends), so a large problem that
    /// improves often never stalls on copies while the latest incumbent still reaches the
    /// stream.
    pub const CAPTURE_INTERVAL: Duration = Duration::from_secs(1);
    /// Retained bytes of the event's own data, the primal included.
    fn bytes(&self) -> usize {
        self.primal
            .as_ref()
            .map_or(0, |p| p.len().saturating_mul(8))
            .saturating_add(64)
    }
}
/// When an adapter captures an incumbent's solution: the first incumbent of an attempt at
/// once, later ones at most once per [`IncumbentEvent::CAPTURE_INTERVAL`]. A deferred
/// capture stays pending until the interval passes or the search ends.
#[derive(Debug)]
pub(crate) struct CaptureThrottle {
    started: Instant,
    /// Nanoseconds after `started` of the last capture; `u64::MAX` before the first.
    last: AtomicU64,
    pending: AtomicBool,
}
impl CaptureThrottle {
    pub(crate) fn new() -> Self {
        Self {
            started: Instant::now(),
            last: AtomicU64::new(u64::MAX),
            pending: AtomicBool::new(false),
        }
    }
    fn now(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_nanos()).unwrap_or(u64::MAX - 1)
    }
    fn due(&self) -> bool {
        let last = self.last.load(Ordering::Acquire);
        last == u64::MAX
            || self.now().saturating_sub(last)
                >= u64::try_from(IncumbentEvent::CAPTURE_INTERVAL.as_nanos()).unwrap_or(u64::MAX)
    }
    /// A new incumbent: whether to capture its solution now. Otherwise the capture is
    /// pending.
    pub(crate) fn admit(&self) -> bool {
        let due = self.due();
        if due {
            self.last.store(self.now(), Ordering::Release);
        }
        self.pending.store(!due, Ordering::Release);
        due
    }
    /// Whether a deferred capture is due now; if so it is taken.
    pub(crate) fn deferred(&self) -> bool {
        if self.pending.load(Ordering::Acquire) && self.due() {
            self.pending.store(false, Ordering::Release);
            self.last.store(self.now(), Ordering::Release);
            return true;
        }
        false
    }
    /// Whether a deferred capture is outstanding when the search ends; if so it is taken.
    pub(crate) fn outstanding(&self) -> bool {
        self.pending.swap(false, Ordering::AcqRel)
    }
}
/// Receives every event pushed to a [`Progress`], whether or not the bounded in-memory
/// stream retains it: a durable stream bounded by its own retention policy instead of an
/// event cap (ADR-0112 Outcome 17).
pub trait ProgressTap: Send + Sync + std::fmt::Debug {
    /// Observe one event. Called on the native thread; it must not block.
    fn observe(&self, event: &Event);
}
/// A bounded event stream shared with an observing handle, optionally tapped by a
/// durable stream.
#[derive(Debug)]
pub struct Progress {
    limit: usize,
    events: Mutex<(Vec<Event>, u64)>,
    tap: Option<Arc<dyn ProgressTap>>,
}
impl Event {
    /// Retained bytes: the phase, the values and the incumbent.
    fn retained_bytes(&self) -> usize {
        self.values
            .iter()
            .fold(self.phase.len().saturating_add(64), |n, (k, v)| {
                n.saturating_add(k.len())
                    .saturating_add(128)
                    .saturating_add(match v {
                        Metric::Text(v) => v.len(),
                        _ => 0,
                    })
            })
            .saturating_add(self.incumbent.as_ref().map_or(0, IncumbentEvent::bytes))
    }
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
            tap: None,
        }
    }
    /// A bounded stream whose every event, retained or dropped here, also reaches `tap`.
    pub fn tapped(limit: usize, tap: Arc<dyn ProgressTap>) -> Self {
        Self {
            tap: Some(tap),
            ..Self::new(limit)
        }
    }
    /// Copy an owned event into the bounded stream, after offering it to the tap. A
    /// retained event holds at most 4 KiB: an incumbent whose solution is larger is
    /// retained without it (the tap received it whole).
    pub fn push(&self, mut event: Event) {
        if let Some(tap) = &self.tap {
            tap.observe(&event);
        }
        if let Ok(mut s) = self.events.lock() {
            if event.retained_bytes() > 4096
                && let Some(incumbent) = event.incumbent.as_mut()
            {
                incumbent.primal = None;
            }
            if s.0.len() < self.limit && event.retained_bytes() <= 4096 {
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
    /// Foreign-library allocation allowance the worker admitted for this attempt, for
    /// adapters whose library enforces its own memory limit; absent when none was admitted.
    pub memory: Option<usize>,
}
impl Execution {
    /// Construct at the admitted worker boundary.
    pub fn new(cancel: Arc<AtomicBool>, controls: &Controls) -> Self {
        Self {
            cancel,
            started: Instant::now(),
            time_limit: controls.time_limit,
            progress: Arc::new(Progress::new(controls.history)),
            memory: None,
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
/// Reuse compatibility separates seed coordinates, the native profile and numeric data
/// (F24).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Compatibility {
    /// Seed coordinate compatibility: source IDs, domains, physical maps and sparse
    /// structure. A seed needs only this and the backend to match, so a step that changes
    /// a native option keeps its predecessor's seed.
    pub layout: ContentHash,
    /// Native profile and session identity: session-relevant controls, typed settings and
    /// the numerical policy. Retained native state needs this to match as well.
    pub profile: ContentHash,
    /// Parameters, fixed values, coefficients and bounds for this attempt.
    pub data: ContentHash,
    /// Native backend.
    pub backend: Backend,
}
impl Compatibility {
    /// Whether native state retained under `self` may serve an attempt stamped `other`:
    /// the same coordinates, profile and backend. Numeric data may differ.
    pub fn same_session(&self, other: &Self) -> bool {
        self.layout == other.layout
            && self.profile == other.profile
            && self.backend == other.backend
    }
}
/// Native basis values retain their original integer codes.
#[derive(Clone, Debug)]
pub struct Basis {
    /// Native column basis statuses.
    pub columns: Vec<i32>,
    /// Native row basis statuses.
    pub rows: Vec<i32>,
}
/// How an interior-point method re-centres a complete primal-dual seed (L-N3). With the
/// cold-start defaults a seeded restart is pushed back towards the analytic centre and loses
/// most of its benefit, so a restart states its barrier and its pushes. Absent fields take
/// these defaults across the Python boundary (ADR-0113).
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
pub struct WarmRestart {
    /// Initial barrier parameter of the restart (`mu_init`).
    pub barrier: RestartBarrier,
    /// Absolute push of the primal seed from its bounds (`warm_start_bound_push`).
    pub bound_push: Tolerance,
    /// Relative push of the primal seed from its bounds (`warm_start_bound_frac`); at most
    /// one half.
    pub bound_frac: Fraction,
    /// Absolute push of the slack seed from its bounds (`warm_start_slack_bound_push`).
    pub slack_bound_push: Tolerance,
    /// Relative push of the slack seed from its bounds (`warm_start_slack_bound_frac`); at
    /// most one half.
    pub slack_bound_frac: Fraction,
    /// Push of the bound multipliers from zero (`warm_start_mult_bound_push`).
    pub mult_bound_push: Tolerance,
}
impl Default for WarmRestart {
    /// The seed's own final barrier and pushes of 1e-9: the restart stays near the seed.
    fn default() -> Self {
        Self {
            barrier: RestartBarrier::Seed,
            bound_push: scalar!(Tolerance(1e-9)),
            bound_frac: scalar!(Fraction(1e-9)),
            slack_bound_push: scalar!(Tolerance(1e-9)),
            slack_bound_frac: scalar!(Fraction(1e-9)),
            mult_bound_push: scalar!(Tolerance(1e-9)),
        }
    }
}
/// The initial barrier parameter of an interior-point restart.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RestartBarrier {
    /// The final barrier value the producing interior-point solve recorded with the seed. A
    /// seed without one (an authored seed) leaves the native default in force, and the
    /// receipt says so.
    Seed,
    /// A stated positive value.
    Value {
        /// The initial barrier parameter.
        value: Tolerance,
    },
}
impl WarmRestart {
    /// The rule the scalar types cannot state: relative pushes inside the native range
    /// (0, 0.5].
    ///
    /// # Errors
    /// A relative push above one half.
    pub fn validate(&self) -> Result<(), ProblemError> {
        if self.bound_frac.into_inner() > 0.5 || self.slack_bound_frac.into_inner() > 0.5 {
            return Err(ProblemError::Contract(
                "warm restart relative pushes must be at most 0.5".into(),
            ));
        }
        Ok(())
    }
    /// The native options of this restart for a seed carrying `barrier` (native
    /// coordinates), and the record of what they set. `mu_init` is set only for the monotone
    /// barrier update, the only one that reads it.
    #[cfg(any(feature = "ipopt", feature = "pounce"))]
    pub(crate) fn apply(&self, barrier: Option<f64>, monotone: bool) -> (Options, AppliedRestart) {
        let mu_init = match self.barrier {
            RestartBarrier::Seed => barrier.filter(|v| v.is_finite() && *v > 0.0),
            RestartBarrier::Value { value } => Some(value.into_inner()),
        }
        .filter(|_| monotone);
        let mut options = Options::from([
            (
                "warm_start_bound_push".into(),
                OptionValue::Real(self.bound_push.into_inner()),
            ),
            (
                "warm_start_bound_frac".into(),
                OptionValue::Real(self.bound_frac.into_inner()),
            ),
            (
                "warm_start_slack_bound_push".into(),
                OptionValue::Real(self.slack_bound_push.into_inner()),
            ),
            (
                "warm_start_slack_bound_frac".into(),
                OptionValue::Real(self.slack_bound_frac.into_inner()),
            ),
            (
                "warm_start_mult_bound_push".into(),
                OptionValue::Real(self.mult_bound_push.into_inner()),
            ),
        ]);
        if let Some(mu) = mu_init {
            options.insert("mu_init".into(), OptionValue::Real(mu));
        }
        (
            options,
            AppliedRestart {
                mu_init,
                profile: *self,
            },
        )
    }
}
/// Native options a [`WarmRestart`] owns; raw options may not set them.
#[cfg(any(feature = "ipopt", feature = "pounce"))]
pub(crate) const RESTART_OPTIONS: [&str; 6] = [
    "mu_init",
    "warm_start_bound_push",
    "warm_start_bound_frac",
    "warm_start_slack_bound_push",
    "warm_start_slack_bound_frac",
    "warm_start_mult_bound_push",
];
/// The interior-point restart an adapter applied to a submitted primal-dual seed.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct AppliedRestart {
    /// Initial barrier parameter set, in native coordinates; `None` left the native default.
    pub mu_init: Option<f64>,
    /// The restart profile that produced it.
    pub profile: WarmRestart,
}
/// A POUNCE active-set working set. It indexes native rows and bounds, so it is meaningful
/// only in the native coordinates of the presolve transformation that produced it (F07).
#[derive(Clone, Debug)]
pub struct WorkingSet {
    /// Native coordinate identity of that transformation (`presolve::Report::transformation`).
    pub transformation: ContentHash,
    /// Native bound and constraint activity.
    #[cfg(feature = "pounce")]
    pub active: pounce_rs::sqp::WorkingSet,
}
/// Starts are typed by their native mathematical meaning.
#[derive(Clone, Debug)]
pub enum WarmPayload {
    /// Primal start, optionally with NLP lower/upper/row multipliers, the final barrier value
    /// of an interior-point producer and an active-set working set.
    Nlp {
        /// Primal in source order.
        primal: Vec<f64>,
        /// Lower/upper bound multipliers.
        bounds: Option<(Vec<f64>, Vec<f64>)>,
        /// Constraint multipliers.
        rows: Option<Vec<f64>>,
        /// Final barrier parameter of the producing interior-point solve, in authored
        /// objective units (native coordinates inside an adapter).
        barrier: Option<f64>,
        /// Active-set working set of the producing SQP solve, keyed by its native
        /// transformation.
        working: Option<WorkingSet>,
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
    pub run: Option<pse_model::generated::identities::RunId>,
    /// Zero-based original attempt.
    pub attempt: usize,
}
impl WarmPayload {
    /// A primal-only NLP start.
    pub fn primal(primal: Vec<f64>) -> Self {
        Self::Nlp {
            primal,
            bounds: None,
            rows: None,
            barrier: None,
            working: None,
        }
    }
    /// Source-coordinate dimensions and finite values of this payload.
    pub fn shaped(&self, variables: usize, rows: usize) -> bool {
        let finite = |v: &[f64], n| v.len() == n && v.iter().all(|v| v.is_finite());
        match self {
            Self::Root(x) => finite(x, variables),
            Self::Nlp {
                primal,
                bounds,
                rows: dual,
                barrier,
                working: _,
            } => {
                // A working set indexes native coordinates; the pipeline checks it against
                // the native transformation.
                finite(primal, variables)
                    && bounds
                        .as_ref()
                        .is_none_or(|(l, u)| finite(l, variables) && finite(u, variables))
                    && dual.as_ref().is_none_or(|d| finite(d, rows))
                    && barrier.is_none_or(|b| b.is_finite() && b > 0.0)
            }
            Self::Highs {
                primal,
                dual,
                basis,
            } => {
                (primal.is_some() || dual.is_some() || basis.is_some())
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
        }
    }
}
impl WarmStart {
    /// Check that the selected adapter consumes this payload variant, and its
    /// source-coordinate shape and finite values, before native construction.
    pub fn validate_shape(&self, variables: usize, rows: usize) -> Result<(), ProblemError> {
        let valid = crate::execution::adapter(self.compatibility.backend).accepts(&self.payload)
            && self.payload.shaped(variables, rows);
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
                barrier,
                working,
            } => {
                serde_json::json!({"kind":"nlp","primal":primal,"bound_duals":bounds,"row_duals":rows,"barrier":barrier,"working_set":working.as_ref().map(WorkingSet::snapshot)})
            }
            WarmPayload::Highs {
                primal,
                dual,
                basis,
            } => {
                serde_json::json!({"kind":"highs","primal":primal,"dual":dual,"basis":basis.as_ref().map(|b|serde_json::json!({"columns":b.columns,"rows":b.rows}))})
            }
        };
        serde_json::json!({"origin":self.origin,"layout":self.compatibility.layout.to_hex(),"profile":self.compatibility.profile.to_hex(),"data":self.compatibility.data.to_hex(),"backend":self.compatibility.backend.as_str(),"payload":payload})
    }
    /// Content identity of this seed (coordinates, profile, data, backend and payload)
    /// without its execution origin, so a lineage identity records what seeded a result
    /// independently of the run that produced the seed (F25).
    pub fn content_key(&self) -> ContentHash {
        let mut detached = self.clone();
        detached.origin = None;
        // Version 2 frames an NLP seed's barrier and working set (DP-24).
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::NativeSeedV2);
        h.str(&detached.snapshot().to_string());
        h.finish_hash()
    }
    /// Numeric data and native profile changes may reuse a seed; its coordinates and
    /// backend must match exactly.
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
impl WorkingSet {
    /// Owned provenance of this working set: every bound and row status by its snake_case
    /// name, never Rust `Debug` text (F30).
    fn snapshot(&self) -> serde_json::Value {
        #[cfg(feature = "pounce")]
        let active = {
            use pounce_rs::sqp::{BoundStatus, ConsStatus};
            let bound = |status: &BoundStatus| match status {
                BoundStatus::Inactive => "inactive",
                BoundStatus::AtLower => "at_lower",
                BoundStatus::AtUpper => "at_upper",
                BoundStatus::Fixed => "fixed",
            };
            let row = |status: &ConsStatus| match status {
                ConsStatus::Inactive => "inactive",
                ConsStatus::AtLower => "at_lower",
                ConsStatus::AtUpper => "at_upper",
                ConsStatus::Equality => "equality",
            };
            serde_json::json!({
                "bounds": self.active.bounds.iter().map(bound).collect::<Vec<_>>(),
                "constraints": self.active.constraints.iter().map(row).collect::<Vec<_>>(),
            })
        };
        #[cfg(not(feature = "pounce"))]
        let active = serde_json::Value::Null;
        serde_json::json!({"transformation": self.transformation.to_hex(), "active": active})
    }
}
/// What became of a submitted working set at the presolve boundary: it reaches the native
/// solver only under the transformation that produced it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct WorkingSetTransfer {
    /// The transformation the working set was produced under.
    pub transformation: ContentHash,
    /// It equals this attempt's transformation and was passed through.
    pub retained: bool,
}
/// One transformation a submitted seed passed through between its source coordinates and
/// the native API, recorded from what actually ran (F25).
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedTransformation {
    /// Model coordinate normalization, by its identity.
    Normalization(ContentHash),
    /// Library presolve: its transformation identity and the passes it applied.
    Presolve {
        /// Native coordinate identity of the projection.
        transformation: ContentHash,
        /// Passes the library actually installed.
        passes: Vec<crate::presolve::Pass>,
    },
    /// An active-set working set checked against the native transformation.
    WorkingSet(WorkingSetTransfer),
    /// The interior-point restart the adapter applied to a primal-dual seed.
    InteriorRestart(AppliedRestart),
}
impl SeedTransformation {
    /// The path a seed takes: normalization, then presolve when the library applied a pass.
    pub fn path(
        normalization: ContentHash,
        preprocessing: Option<&crate::presolve::Report>,
    ) -> Vec<Self> {
        let mut path = vec![Self::Normalization(normalization)];
        if let Some(report) = preprocessing {
            let passes: Vec<_> = report
                .passes
                .iter()
                .filter(|(_, p)| p.applied)
                .map(|(pass, _)| *pass)
                .collect();
            if !passes.is_empty() {
                path.push(Self::Presolve {
                    transformation: report.transformation,
                    passes,
                });
            }
            if let Some(transfer) = report.working_set {
                path.push(Self::WorkingSet(transfer));
            }
        }
        path
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
    /// Transformations the seed passed through before the native API.
    pub transformations: Vec<SeedTransformation>,
    /// API submission is observable; native internal consumption may remain unavailable.
    pub submitted: bool,
}
impl StartReceipt {
    /// Complete this receipt from the attempt that consumed its seed: whether the native API
    /// received it and every transformation it passed through, including the interior-point
    /// restart the adapter applied (F25, L-N3).
    pub fn record(&mut self, report: &SolveReport, normalization: ContentHash) {
        self.submitted = report.evidence.start_submitted;
        if self.seed.is_some() || self.sparse_seed.is_some() {
            self.transformations =
                SeedTransformation::path(normalization, report.preprocessing.as_ref());
            if let Some(restart) = report.evidence.restart {
                self.transformations
                    .push(SeedTransformation::InteriorRestart(restart));
            }
        }
    }
    /// Owned JSON provenance of this receipt, shared by publication and the Python surface.
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "previous_attempt": self.previous_attempt,
            "seed": self.seed.as_ref().map(WarmStart::snapshot),
            "sparse_seed": self.sparse_seed.as_ref().map(|s| s
                .iter()
                .map(|(id, v)| (id.to_hex(), *v))
                .collect::<BTreeMap<_, _>>()),
            "transformations": self.transformations,
            "submitted": self.submitted,
        })
    }
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
/// Native solution status as reported by the library, never inferred from values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolutionStatus {
    /// The library supplied no solution of this kind.
    Unavailable,
    /// A solution was supplied but the library reports it infeasible.
    Infeasible,
    /// The library reports a feasible solution.
    Feasible,
}
/// Callback history consumed by retry policy, independent of bounded event retention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CallbackEvidence {
    /// Recoverable trial refusals.
    pub trial_rejections: usize,
    /// A terminal failure latched and ended the attempt.
    pub terminal_failure: bool,
}
/// Post-solve curvature at an NLP candidate (L-N6, PS-12): the certified inertia of the
/// normalized KKT matrix over the free variables and the active constraints of the decisive
/// test, what it decides, and its 1-norm condition estimate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SecondOrder {
    /// Variables not held at a bound by the tested constraint set.
    pub free: usize,
    /// Active rows and bounds in the tested set.
    pub active: usize,
    /// Active rows or bounds whose normalized multiplier is within the dual budget of zero;
    /// strict complementarity fails for them.
    pub weakly_active: usize,
    /// Certified inertia (positive, negative, zero) of the tested KKT matrix.
    pub inertia: (usize, usize, usize),
    /// Hager–Higham estimate of its 1-norm condition number, a lower bound.
    pub condition_1norm: Option<f64>,
    /// What the inertia decides.
    pub curvature: Curvature,
}
/// The verdict of the post-solve second-order check, in the minimization convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Curvature {
    /// The reduced Hessian is positive definite on the null space of the strongly active
    /// constraints, which contains the critical cone: with independent active gradients the
    /// candidate is a strict local minimizer (second-order sufficient conditions).
    Sufficient,
    /// A direction satisfying every active constraint has negative curvature: the candidate
    /// is not a local minimizer.
    NegativeCurvature,
    /// A zero eigenvalue: the reduced Hessian is singular or the active gradients are
    /// dependent.
    Singular,
    /// Weakly active constraints leave the critical cone between the two subspaces tested,
    /// and neither test decides.
    Undecided,
}
/// Original-coordinate KKT checks; `None` means the measure was unavailable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KktEvidence {
    /// Normalized stationarity within its budget.
    pub stationarity: Option<bool>,
    /// Normalized complementarity within its budget.
    pub complementarity: Option<bool>,
}
/// Coefficient-model evidence reported by the `HiGHS` adapter, in normalized coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoefficientEvidence {
    /// The native model was read back and equals the admitted coefficient model.
    pub upload_equivalent: bool,
    /// The model has integer or semi-continuous columns.
    pub discrete: bool,
    /// Native objective at the returned primal.
    pub objective: Option<f64>,
    /// Native relative MIP gap.
    pub mip_gap: Option<f64>,
    /// Native MIP dual bound.
    pub mip_dual_bound: Option<f64>,
    /// Native primal solution status.
    pub primal: SolutionStatus,
    /// Native dual solution status.
    pub dual: SolutionStatus,
    /// Native maximum dual infeasibility.
    pub max_dual_infeasibility: Option<f64>,
    /// Native primal-dual objective error.
    pub primal_dual_objective_error: Option<f64>,
}
/// Conic residuals reported by the Clarabel adapter, in normalized coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConicEvidence {
    /// Primal residual.
    pub primal_residual: f64,
    /// Dual residual.
    pub dual_residual: f64,
    /// Absolute duality gap.
    pub gap_absolute: f64,
    /// Relative duality gap.
    pub gap_relative: f64,
}
/// Where a global dual bound comes from (ADR-0106 §10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundSource {
    /// The backend's bound over an exact export of the original program.
    ExactExport,
    /// The backend's bound over a sound relaxation of the original program: valid for
    /// bounds and infeasibility conclusions, never for a solution claim (ADR-0105 §2).
    RelaxedExport,
}
/// Where the reported primal candidate comes from (ADR-0105 §2, T07).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimalSource {
    /// The backend's incumbent over an exact export, re-qualified in original coordinates.
    Backend,
    /// The backend's incumbent over a relaxed export: an assignment proposal and an
    /// observation only, never a result or a seed.
    RelaxedIncumbent,
    /// The continuous re-solve with the backend's discrete assignment fixed, run through
    /// the one NLP runner and qualified in original coordinates.
    FixedAssignment,
}
/// Evidence of a certifying adapter over an exported factorable program (ADR-0106 §10).
/// Bounds are in the authored objective sense and original coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlobalEvidence {
    /// Worst fidelity of the exported rows, constraints and objective.
    pub fidelity: pse_math::factorable::Fidelity,
    /// Identity of the declared box the backend branched over.
    pub domain: ContentHash,
    /// Authored objective orientation of the bounds.
    pub sense: pse_math::binding::ObjectiveSense,
    /// Native feasibility tolerance the bounds hold within.
    pub feasibility: f64,
    /// Requested relative gap.
    pub gap_relative: f64,
    /// Requested absolute gap, in original objective units.
    pub gap_absolute: f64,
    /// Native dual bound, when finite.
    pub dual_bound: Option<f64>,
    /// Native primal bound, when finite.
    pub primal_bound: Option<f64>,
    /// Native relative gap, when finite.
    pub gap: Option<f64>,
    /// Branch-and-bound nodes, including restarts.
    pub nodes: i64,
    /// The native model was read back and evaluates the exported functions as the neutral
    /// program does.
    pub readback: bool,
    /// Source of the dual bound.
    pub dual: BoundSource,
    /// Source of the reported candidate.
    pub primal: PrimalSource,
    /// The backend concluded that the exported program is infeasible over the box.
    pub infeasible: bool,
    /// Optimality or infeasibility was established in rational arithmetic (exact MILP).
    pub exact: bool,
}
/// Bulky records of a certifying adapter beside the decision evidence: the declared box,
/// the ranked solution pool, an infeasible subsystem and an exact objective. Nothing here
/// grants a claim; [`GlobalEvidence`] carries the decision inputs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlobalRecord {
    /// The declared closed box the backend branched over: program columns in order, then
    /// auxiliaries.
    pub boxes: Vec<(f64, f64)>,
    /// Ranked solutions the backend stored, best first, each re-qualified in original
    /// coordinates by the runner.
    pub pool: Vec<PoolSolution>,
    /// Infeasible subsystem of the exported program, when requested after a proof of
    /// infeasibility.
    pub iis: Option<Iis>,
    /// Exact rational objective of the reported solution, as native text.
    pub exact_objective: Option<String>,
    /// The attempt reused a retained native search tree (reoptimization).
    pub reoptimized: bool,
}
/// One ranked solution of the backend's pool.
#[derive(Clone, Debug, PartialEq)]
pub struct PoolSolution {
    /// Rank in the backend's ordering, zero first.
    pub rank: usize,
    /// Values in program column order.
    pub primal: Vec<f64>,
    /// Native objective in the authored sense, when an objective is exported.
    pub objective: Option<f64>,
    /// Original-coordinate feasibility; absent when it could not be evaluated.
    pub feasible: Option<bool>,
}
/// An infeasible subsystem of the exported program (ADR-0105 §8): exported functions and
/// declared bounds whose conjunction the backend proved infeasible over the box. The
/// functions are minimized; the declared bounds of the variables they use are kept.
#[derive(Clone, Debug, PartialEq)]
pub struct Iis {
    /// The backend reports the subsystem irreducible: removing any function member leaves
    /// a feasible system over the kept bounds, within its tolerances.
    pub irreducible: bool,
    /// Members in a deterministic order.
    pub members: Vec<IisMember>,
}
/// One member of an infeasible subsystem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IisMember {
    /// A selected row.
    Row(SemanticId),
    /// An unconditional obligation's closed constraint.
    Obligation {
        /// Bound instance.
        instance: SemanticId,
        /// Authored occurrence.
        source: SemanticId,
    },
    /// An implicit block's residual.
    Residual {
        /// Instance evaluating the block.
        instance: SemanticId,
        /// Residual ordinal.
        ordinal: usize,
    },
    /// An implicit block's declared bound.
    ImplicitBound {
        /// Instance evaluating the block.
        instance: SemanticId,
        /// Bound ordinal.
        ordinal: usize,
    },
    /// A native constraint, by its ordinal in the program.
    Native(usize),
    /// A variable's lower bound.
    VariableLower(SemanticId),
    /// A variable's upper bound.
    VariableUpper(SemanticId),
    /// An auxiliary's lower bound, by auxiliary ordinal.
    AuxiliaryLower(usize),
    /// An auxiliary's upper bound, by auxiliary ordinal.
    AuxiliaryUpper(usize),
}
/// Typed adapter evidence. Qualification, retry and start receipts read only this;
/// metrics remain observations and are never an input to a decision.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Evidence {
    /// Callback trial history.
    pub callback: CallbackEvidence,
    /// A start was submitted through the native API.
    pub start_submitted: bool,
    /// Native state retained from an earlier step was reused by this attempt.
    pub reused_native_state: bool,
    /// Interior-point restart applied to the submitted primal-dual seed.
    pub restart: Option<AppliedRestart>,
    /// An active-set working set reached the native solver.
    pub working_set_submitted: bool,
    /// Original-coordinate KKT acceptance, recorded by `quality::record_kkt`.
    pub kkt: Option<KktEvidence>,
    /// Coefficient-model evidence.
    pub coefficient: Option<CoefficientEvidence>,
    /// Conic residual evidence.
    pub conic: Option<ConicEvidence>,
    /// Post-solve second-order check of an optimizing NLP candidate.
    pub second_order: Option<SecondOrder>,
    /// Global bound evidence of a certifying adapter.
    pub global: Option<GlobalEvidence>,
}
/// The label of a candidate that minimizes constraint violation instead of satisfying the
/// constraints: the point a local infeasibility stop returns, including the ℓ1 exact-penalty
/// route's (ADR-0109 item 3). It is diagnostic evidence only (ADR-0106 `diagnostic_only`),
/// never a result, and names the rows it leaves violated.
#[derive(Clone, Debug)]
pub struct LeastInfeasible {
    /// Original rows violated beyond their acceptance budget, with the physical violation.
    pub violated: Vec<crate::quality::Violation>,
}
/// One native attempt, including unsuccessful attempts with no usable candidate.
#[derive(Clone, Debug)]
pub struct SolveReport {
    pub(crate) callback_failure: Option<Arc<ProblemError>>,
    validation_failure: Option<Arc<ProblemError>>,
    /// Typed adapter evidence consumed by qualification and retry.
    pub evidence: Evidence,
    failure_owner: Option<Arc<dyn pse_math::AllocationOwner>>,
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
    /// Records of a certifying adapter: box, solution pool, IIS, exact objective.
    pub global: Option<Arc<GlobalRecord>>,
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
    /// Independent original-space checks; absent when validation failed or no candidate exists.
    pub quality: Option<crate::quality::Quality>,
    /// Native and adapter observations; absent means unavailable, never implicitly zero.
    /// Decisions read `evidence`, never these keys.
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
    /// Set when the candidate is a least-infeasible point.
    pub least_infeasible: Option<LeastInfeasible>,
}
impl SolveReport {
    /// Original typed cause of a failed native evaluation, independent of event retention.
    pub fn callback_failure(&self) -> Option<&ProblemError> {
        self.callback_failure.as_deref()
    }
    /// Typed failure from independent original-model observation after native exit.
    /// It is the only record of that failure; messages are derived from it.
    pub fn validation_failure(&self) -> Option<&ProblemError> {
        self.validation_failure.as_deref()
    }
    /// Record why independent validation failed. Assurance is withdrawn; the native
    /// termination is preserved.
    pub fn record_validation_failure(&mut self, error: ProblemError) {
        self.validation_failure = Some(Arc::new(error));
        self.termination.assurance = Assurance::None;
    }
    pub(crate) fn clear_validation_failure(&mut self) {
        self.validation_failure = None;
    }
    /// Variable retained failure extent, including shared-pointer allocation overhead.
    pub fn failure_bytes(&self) -> usize {
        [self.callback_failure(), self.validation_failure()]
            .into_iter()
            .flatten()
            .fold(0usize, |bytes, e| {
                bytes.saturating_add(e.retained_bytes()).saturating_add(128)
            })
    }
    /// Attach the runtime reservation to every clone of this owned report.
    pub fn with_failure_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.failure_owner = Some(owner);
        self
    }
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
            callback_failure: None,
            validation_failure: None,
            evidence: Evidence::default(),
            failure_owner: None,
            owner: None,
            observation: None,
            preprocessing: None,
            certificate: None,
            global: None,
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
            metrics: BTreeMap::new(),
            options: Options::new(),
            native_defaults: Options::new(),
            provenance: BTreeMap::new(),
            events,
            dropped_events,
            warm_start: None,
            start_receipt: None,
            qualification: Qualification::Unqualified,
            least_infeasible: None,
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
    fn identity_covers_every_settings_field() {
        let base = Controls::default();
        let key = base.identity().unwrap();
        // Exhaustive: a new control fails to compile until it is covered here.
        let Controls {
            time_limit,
            iterations,
            threads,
            history,
            hessian: _,
            reuse: _,
            start: _,
            options: _,
        } = base.clone();
        let variants = [
            Controls {
                time_limit: time_limit + Duration::from_nanos(1),
                ..base.clone()
            },
            Controls {
                iterations: iterations + 1,
                ..base.clone()
            },
            Controls {
                threads: threads + 1,
                ..base.clone()
            },
            Controls {
                history: history + 1,
                ..base.clone()
            },
            Controls {
                hessian: HessianMode::LimitedMemory,
                ..base.clone()
            },
            Controls {
                reuse: ReusePolicy::AllowRebuild,
                ..base.clone()
            },
            Controls {
                start: StartPolicy::PreviousAccepted,
                ..base.clone()
            },
            Controls {
                options: Options::from([("mu_init".into(), OptionValue::Real(0.1))]),
                ..base.clone()
            },
        ];
        for variant in &variants {
            assert_ne!(variant.identity().unwrap(), key, "{variant:?}");
        }
        // Option values keep their native type and exact float bits.
        let option = |v| {
            Controls {
                options: Options::from([("x".into(), v)]),
                ..base.clone()
            }
            .identity()
            .unwrap()
        };
        assert_ne!(
            option(OptionValue::Real(0.0)),
            option(OptionValue::Real(-0.0))
        );
        assert_ne!(
            option(OptionValue::Integer(1)),
            option(OptionValue::Real(1.0))
        );
        assert_ne!(
            option(OptionValue::Bool(true)),
            option(OptionValue::Text("yes".into()))
        );
        // Every resolved budget enters its identity as well.
        let accuracy = ResolvedAccuracy::nominal();
        let key = accuracy.key().unwrap();
        let ResolvedAccuracy {
            feasibility,
            stationarity,
            complementarity,
            integrality,
            gap_absolute,
            gap_relative,
            mip_absolute_gap,
            mip_relative_gap,
            acceptable: _,
            native_scaling,
        } = accuracy.clone();
        for variant in [
            ResolvedAccuracy {
                feasibility: feasibility * 2.0,
                ..accuracy.clone()
            },
            ResolvedAccuracy {
                stationarity: stationarity * 2.0,
                ..accuracy.clone()
            },
            ResolvedAccuracy {
                complementarity: complementarity * 2.0,
                ..accuracy.clone()
            },
            ResolvedAccuracy {
                integrality: integrality * 2.0,
                ..accuracy.clone()
            },
            ResolvedAccuracy {
                gap_absolute: gap_absolute * 2.0,
                ..accuracy.clone()
            },
            ResolvedAccuracy {
                gap_relative: gap_relative * 2.0,
                ..accuracy.clone()
            },
            ResolvedAccuracy {
                mip_absolute_gap: mip_absolute_gap * 2.0,
                ..accuracy.clone()
            },
            ResolvedAccuracy {
                mip_relative_gap: mip_relative_gap * 2.0,
                ..accuracy.clone()
            },
            ResolvedAccuracy {
                acceptable: Some(pse_model::numerics::KktTolerances {
                    stationarity: 1e-5,
                    complementarity: 1e-5,
                }),
                ..accuracy.clone()
            },
            ResolvedAccuracy {
                native_scaling: !native_scaling,
                ..accuracy.clone()
            },
        ] {
            assert_ne!(variant.key().unwrap(), key, "{variant:?}");
        }
    }
    #[test]
    fn numerical_options_keep_feasibility_kkt_and_acceptable_independent() {
        let mut accuracy = ResolvedAccuracy {
            feasibility: 1e-7,
            stationarity: 2e-8,
            complementarity: 3e-9,
            ..ResolvedAccuracy::nominal()
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
        let key = accuracy.key().unwrap();
        accuracy.acceptable = Some(pse_model::numerics::KktTolerances {
            stationarity: 1e-5,
            complementarity: 1e-6,
        });
        assert_ne!(key, accuracy.key().unwrap());
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
        let resolved = ResolvedAccuracy::resolve(&Default::default(), &t, &scales).unwrap();
        assert!((resolved.feasibility - 1e-7).abs() < 1e-20);
    }
}
