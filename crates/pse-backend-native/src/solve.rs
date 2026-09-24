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

/// Native implementation identity, never inferred from a status integer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Backend {
    /// Direct Ipopt C interface.
    Ipopt,
    /// Native Rust TNLP interface.
    Pounce,
    /// SUNDIALS square nonlinear systems.
    Kinsol,
    /// Linear, mixed-integer and convex quadratic optimization.
    Highs,
    /// Explicit continuous convex cones.
    Clarabel,
}
impl Backend {
    /// Explicit representational and execution capabilities of the linked adapters.
    /// Eligibility still requires admission of the actual model and profile.
    pub const fn capabilities(self) -> Capabilities {
        match self {
            Self::Ipopt => Capabilities {
                classes: &[ProblemClass::SmoothNlp],
                derivatives: DerivativeCapability::ExactHessianOrLimitedMemory,
                warm: WarmCapability::PrimalDual,
                reuse: "same sparse layout and bounds: retained C problem",
                cancellation: "intermediate/evaluation checkpoints",
                diagnostics: "native current iterate, violations, callback counts and timing",
            },
            Self::Pounce => Capabilities {
                classes: &[ProblemClass::SmoothNlp],
                derivatives: DerivativeCapability::ExactHessianOrLimitedMemory,
                warm: WarmCapability::PrimalDualAndWorkingSet,
                reuse: "native application and compatible starts; iteration factors are library-owned",
                cancellation: "TNLP intermediate/evaluation checkpoints",
                diagnostics: "complete SolveStatistics, phase timing, FERAL inertia/pivots/fill, restoration and crossover",
            },
            Self::Kinsol => Capabilities {
                classes: &[ProblemClass::SquareRoot, ProblemClass::DeclaredFixedPoint],
                derivatives: DerivativeCapability::JacobianOrProduct,
                warm: WarmCapability::Primal,
                reuse: "same sparse layout: retained SUNDIALS/KLU allocations",
                cancellation: "evaluation checkpoints; native factorization completes before teardown",
                diagnostics: "native nonlinear/linear iterations, setups, failures, norms and callback timing",
            },
            Self::Highs => Capabilities {
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
                classes: &[ProblemClass::ContinuousCone],
                derivatives: DerivativeCapability::Coefficients,
                warm: WarmCapability::None,
                reuse: "native data-update eligibility; reusable mode disables preprocessing",
                cancellation: "native iteration termination callback",
                diagnostics: "complete native info/settings, cone slacks/duals and certificates",
            },
        }
    }
    /// Whether this binary includes the adapter and its native link profile.
    pub const fn available(self) -> bool {
        match self {
            Self::Ipopt => cfg!(feature = "ipopt"),
            Self::Pounce => cfg!(feature = "pounce"),
            Self::Kinsol => cfg!(feature = "kinsol"),
            Self::Highs => cfg!(feature = "highs"),
            Self::Clarabel => true,
        }
    }
}
/// Mathematical families exposed by an adapter, independent of model eligibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProblemClass {
    /// Smooth continuous constrained optimization.
    SmoothNlp,
    /// Square equality system.
    SquareRoot,
    /// Explicit causal map or linear splitting.
    DeclaredFixedPoint,
    /// Affine continuous optimization.
    Linear,
    /// Affine integer, binary and semi-domain optimization.
    MixedLinear,
    /// Certified convex continuous quadratic optimization.
    ConvexQuadratic,
    /// Explicit continuous product of convex cones.
    ContinuousCone,
}
/// Required mathematical derivative representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DerivativeCapability {
    /// Exact first derivatives and exact or native quasi-Newton second order.
    ExactHessianOrLimitedMemory,
    /// Analytic matrix or matrix-vector product.
    JacobianOrProduct,
    /// Explicit coefficient matrices.
    Coefficients,
}
/// Native externally supplied seed support; no absent interface is simulated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarmCapability {
    /// No external iterate API.
    None,
    /// Initial coordinates.
    Primal,
    /// Initial primal and multiplier coordinates.
    PrimalDual,
    /// Primal/dual coordinates and native active working set.
    PrimalDualAndWorkingSet,
    /// Primal/dual coordinates and native simplex basis.
    PrimalDualAndBasis,
}
/// Inspectable adapter contract with explicit operational limitations.
#[derive(Clone, Copy, Debug)]
pub struct Capabilities {
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
/// Finite shared attempt controls; solver-specific settings remain native typed values.
#[derive(Clone, Debug)]
pub struct Controls {
    /// Positive wall-clock allowance, including callbacks.
    pub time_limit: Duration,
    /// Positive native iteration limit.
    pub iterations: u32,
    /// Positive dimensionless acceptance tolerance.
    pub tolerance: f64,
    /// Explicit admitted native thread count.
    pub threads: usize,
    /// Bounded retained progress and failure events.
    pub history: usize,
    /// Hessian representation for compatible NLP methods.
    pub hessian: HessianMode,
    /// Native allocation/data reuse policy.
    pub reuse: ReusePolicy,
    /// Additional native options, admitted by the selected adapter.
    pub options: Options,
}
impl Default for Controls {
    fn default() -> Self {
        Self {
            time_limit: Duration::from_secs(300),
            iterations: 3000,
            tolerance: 1e-8,
            threads: 1,
            history: 256,
            hessian: HessianMode::Exact,
            reuse: ReusePolicy::Fresh,
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
            || !self.tolerance.is_finite()
            || self.tolerance <= 0.0
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
/// Result assurance is independent of native termination and candidate availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Assurance {
    /// No mathematical success established.
    None,
    /// Original residual acceptance for a root or feasible point.
    Feasible,
    /// Local first-order NLP result, not a global optimum.
    LocalStationary,
    /// Native convex/linear/MIP optimality claim.
    NativeOptimal,
    /// Native primal/dual infeasibility certificate is supplied.
    Certificate,
}
/// Cross-backend termination category; the raw code/name are always retained too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Termination {
    /// Library success.
    Success,
    /// Relaxed native convergence criterion.
    Acceptable,
    /// Feasible point without optimality.
    FeasibleOnly,
    /// Native infeasibility detection; scope depends on backend.
    Infeasible,
    /// Native unboundedness detection.
    Unbounded,
    /// Native ambiguous infeasibility/unboundedness.
    InfeasibleOrUnbounded,
    /// Iteration/node/solution limit.
    Limit,
    /// Wall/CPU limit.
    TimeLimit,
    /// Explicit cancellation.
    Cancelled,
    /// Numerical, factorization or restoration failure.
    Numerical,
    /// Terminal evaluator failure.
    Evaluation,
    /// Panic contained at the foreign boundary.
    Panic,
    /// Invalid native configuration or unsupported status.
    Invalid,
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
                s.0.push(event)
            } else {
                s.1 = s.1.saturating_add(1)
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
    /// HiGHS primal/dual start and optional simplex basis.
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
    /// Compatibility evidence.
    pub compatibility: Compatibility,
    /// Native-class seed.
    pub payload: WarmPayload,
}
impl WarmStart {
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
                Metric::Text("unavailable_or_nonfinite".into()),
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
