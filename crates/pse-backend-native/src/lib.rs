// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Class-specific native problem contracts. Execution adapters are supplied by Plan 14 M11–M17.
pub mod assembled;
pub mod callback;
mod certificate;
pub mod conditioning;
pub mod conic;
mod convexity;
pub mod derivative_diagnostics;
pub mod dynamics;
pub mod execution;
#[cfg(feature = "highs")]
pub mod highs;
pub mod identity;
#[cfg(feature = "kinsol")]
pub mod implicit;
#[cfg(feature = "ipopt")]
pub mod ipopt;
#[cfg(feature = "highs")]
pub mod jacobian_diagnostics;
#[cfg(feature = "kinsol")]
pub mod kinsol;
pub mod kkt;
#[cfg(any(feature = "ipopt", feature = "clarabel-pardiso"))]
mod mkl;
mod nlp_pattern;
#[cfg(feature = "pounce")]
pub mod pounce;
pub mod presolve;
pub mod quality;
#[cfg(feature = "kinsol")]
pub mod recycle;
pub mod routing;
#[cfg(feature = "scip")]
pub mod scip;
pub mod settings;
pub mod solve;
/// Original-equation structural admission.
pub mod structural;
pub mod tears;
mod tnlp;
pub mod transform;
pub mod transport;
pub use convexity::GramCertificate;
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::DerivativeOrder;

#[cfg(any(feature = "kinsol", feature = "idas"))]
#[expect(
    unsafe_code,
    reason = "one linked SUNDIALS query writes bounded caller-owned version storage"
)]
fn sundials_version() -> String {
    let (mut major, mut minor, mut patch) = (0, 0, 0);
    let mut label = [0_i8; 128];
    // SAFETY: all output pointers reference writable, correctly sized storage.
    let code = unsafe {
        sundials_sys::SUNDIALSGetVersionNumber(
            &mut major,
            &mut minor,
            &mut patch,
            label.as_mut_ptr(),
            128,
        )
    };
    if code == 0 {
        format!("SUNDIALS {major}.{minor}.{patch}")
    } else {
        format!("SUNDIALS version unavailable ({code})")
    }
}
#[cfg(test)]
use std::sync::{Arc, atomic::AtomicBool};

/// Native status that ended a library operation, retaining its identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeStatus {
    /// Library that reported the status.
    pub backend: solve::Backend,
    /// Unmodified native code.
    pub code: i64,
    /// Native symbolic name.
    pub name: String,
}
impl std::fmt::Display for NativeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} ({})", self.backend.as_str(), self.name, self.code)
    }
}
/// Finite allowance whose exhaustion ended an operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitKind {
    /// Wall-clock deadline.
    Time,
    /// Native iteration, step or evaluation allowance.
    Work,
    /// Memory or native allocation.
    Memory,
}
fn native_suffix(status: Option<&NativeStatus>) -> String {
    status.map_or_else(String::new, |s| format!(" [{s}]"))
}
/// Why retained native state cannot serve a step whose policy requires reusing it (PS-10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReuseRefusal {
    /// Another backend holds the retained state.
    Foreign(solve::Backend),
    /// The step's coordinates, profile, sparsity, bounds or settings differ from those of
    /// the retained state, which must be rebuilt for them.
    Structure,
    /// The retained native problem holds these option keys and the step sets none of them.
    /// The native interface cannot unset an option, so reuse would carry their earlier
    /// values into the step (F02).
    DroppedOptions(Vec<String>),
}
impl std::fmt::Display for ReuseRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Foreign(held) => write!(f, "{} holds the retained state", held.as_str()),
            Self::Structure => f.write_str("coordinates, sparsity, bounds or settings changed"),
            Self::DroppedOptions(keys) => {
                write!(f, "the step drops retained options {}", keys.join(", "))
            }
        }
    }
}
/// A refused request or an attributable failure, classified by its cause (DP-21).
#[derive(Debug, thiserror::Error)]
pub enum ProblemError {
    /// The selected backend is not linked; no implicit fallback is performed.
    #[error(
        "selected backend {backend:?} is unavailable; available alternatives: {alternatives:?}"
    )]
    Unavailable {
        /// Requested implementation.
        backend: solve::Backend,
        /// Other linked backends; eligibility still requires admission.
        alternatives: Vec<solve::Backend>,
    },
    /// The model or request violates a declared contract.
    #[error("invalid native problem: {0}")]
    Contract(String),
    /// Library matching found deficient original equality support.
    #[error("{mode:?} structural deficiency: rows={rows:?}, columns={columns:?}")]
    Structural {
        /// Selected mathematical class.
        mode: structural::Mode,
        /// Equality rows in an overdetermined region.
        rows: Vec<SemanticId>,
        /// Free variables in an underdetermined root region.
        columns: Vec<SemanticId>,
    },
    /// Mathematical evaluation retains its domain/provider cause.
    #[error(transparent)]
    Math(#[from] pse_math::MathError),
    /// A registered provider failed outside an attributed expression; the typed cause is kept.
    #[error("provider failure: {0}")]
    Provider(#[from] pse_kernels::ProviderError),
    /// No eligible route, or the selected adapter cannot represent the request.
    #[error("unsupported: {0}")]
    Unsupported(String),
    /// The step requires reusing retained native state that cannot serve it.
    #[error("{} cannot reuse its retained native state: {refusal}", .backend.as_str())]
    Reuse {
        /// Backend whose reuse was required.
        backend: solve::Backend,
        /// Why the retained state cannot serve the step.
        refusal: ReuseRefusal,
    },
    /// A native method or numerical kernel failed; the native status is kept when one exists.
    #[error("numerical failure: {detail}{}", native_suffix(.status.as_ref()))]
    Numerical {
        /// Native status of the failed library operation, when it reported one.
        status: Option<NativeStatus>,
        /// Failed operation.
        detail: String,
    },
    /// A declared finite allowance was exhausted.
    #[error("{kind:?} limit: {detail}")]
    Limit {
        /// Exhausted allowance.
        kind: LimitKind,
        /// Exhausted operation.
        detail: String,
    },
    /// Cooperative cancellation fired.
    #[error("native work cancelled")]
    Cancelled,
    /// An adapter postcondition or platform invariant failed.
    #[error("internal invariant: {0}")]
    Internal(String),
}
impl ProblemError {
    /// No eligible route, or an adapter that cannot represent the request.
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported(message.into())
    }
    /// An adapter postcondition or platform invariant failed.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
    /// A numerical kernel failed without a native status.
    pub fn numerical(detail: impl Into<String>) -> Self {
        Self::Numerical {
            status: None,
            detail: detail.into(),
        }
    }
    /// A native allocation or memory allowance failed.
    pub fn memory(detail: impl Into<String>) -> Self {
        Self::Limit {
            kind: LimitKind::Memory,
            detail: detail.into(),
        }
    }
    /// A cooperative stop observed at an execution checkpoint.
    pub fn stopped(stop: solve::Termination, detail: impl Into<String>) -> Self {
        match stop {
            solve::Termination::Cancelled => Self::Cancelled,
            solve::Termination::TimeLimit => Self::Limit {
                kind: LimitKind::Time,
                detail: detail.into(),
            },
            _ => Self::Internal(detail.into()),
        }
    }
    /// Classify a failed native call by its mapped status category. Evaluation
    /// failures are attributed by the callback owner, which holds the typed cause.
    pub fn native(
        status: NativeStatus,
        category: solve::Termination,
        detail: impl Into<String>,
    ) -> Self {
        use solve::Termination as T;
        let detail = detail.into();
        match category {
            T::Cancelled => Self::Cancelled,
            T::TimeLimit => Self::Limit {
                kind: LimitKind::Time,
                detail: format!("{detail} [{status}]"),
            },
            T::IterationLimit | T::Limit | T::SolutionLimit | T::ObjectiveLimit => Self::Limit {
                kind: LimitKind::Work,
                detail: format!("{detail} [{status}]"),
            },
            T::ResourceExhausted => Self::Limit {
                kind: LimitKind::Memory,
                detail: format!("{detail} [{status}]"),
            },
            T::Invalid | T::Panic => Self::Internal(format!("{detail} [{status}]")),
            T::Success
            | T::Acceptable
            | T::FeasibleOnly
            | T::Infeasible
            | T::Unbounded
            | T::InfeasibleOrUnbounded
            | T::Inconclusive
            | T::Numerical
            | T::Evaluation => Self::Numerical {
                status: Some(status),
                detail,
            },
        }
    }
    /// Owned error payload admitted separately from bounded native report history.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(match self {
            Self::Unavailable { alternatives, .. } => alternatives
                .capacity()
                .saturating_mul(size_of::<solve::Backend>()),
            Self::Contract(s) | Self::Unsupported(s) | Self::Internal(s) => s.capacity(),
            Self::Numerical { status, detail } => detail
                .capacity()
                .saturating_add(status.as_ref().map_or(0, |s| s.name.capacity())),
            Self::Limit { detail, .. } => detail.capacity(),
            Self::Reuse { refusal, .. } => match refusal {
                ReuseRefusal::DroppedOptions(keys) => keys.iter().fold(
                    keys.capacity().saturating_mul(size_of::<String>()),
                    |bytes, key| bytes.saturating_add(key.capacity()),
                ),
                ReuseRefusal::Foreign(_) | ReuseRefusal::Structure => 0,
            },
            Self::Structural { rows, columns, .. } => rows
                .capacity()
                .saturating_add(columns.capacity())
                .saturating_mul(size_of::<SemanticId>()),
            Self::Math(e) => e.retained_bytes(),
            Self::Provider(e) => e.retained_bytes(),
            Self::Cancelled => 0,
        })
    }
}
pse_diagnostics::impl_diagnostic! {
    ProblemError,
    code(this) { match this {
        Self::Contract(_) | Self::Structural{..} => Some(pse_diagnostics::DiagnosticCode::CompileMath),
        Self::Unavailable{..} | Self::Unsupported(_) | Self::Reuse{..} => Some(pse_diagnostics::DiagnosticCode::CapabilityBackend),
        Self::Numerical{..} => Some(pse_diagnostics::DiagnosticCode::SolveSolverError),
        Self::Limit{kind: LimitKind::Time, ..} => Some(pse_diagnostics::DiagnosticCode::RuntimeTimeout),
        Self::Limit{..} => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        Self::Cancelled => Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
        Self::Internal(_) => Some(pse_diagnostics::DiagnosticCode::InternalInvariant),
        Self::Math(_) | Self::Provider(_) => None,
    } },
    forward(this) { match this { Self::Math(error) => Some(error), Self::Provider(error) => Some(error), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
/// Stable source identity and interval in its declared source representation.
#[derive(Clone, Debug)]
pub struct Variable {
    /// Original physical variable.
    pub id: SemanticId,
    /// Lower bound; negative infinity is permitted.
    pub lower: f64,
    /// Upper bound; positive infinity is permitted.
    pub upper: f64,
}
/// Physical validity, mathematical smoothness and implemented derivatives are separate claims.
#[derive(Clone, Debug)]
pub struct OracleContract {
    /// Exact admitted structure and physical interpretation.
    pub identity: ContentHash,
    /// Independent source variables, in native column order.
    pub variables: Vec<Variable>,
    /// Stable residual/constraint rows.
    pub rows: Vec<SemanticId>,
    /// Available exact local derivative order.
    pub derivatives: DerivativeOrder,
    /// Neighborhood smoothness established by admission, including the selected phase.
    pub smoothness: DerivativeOrder,
}
impl OracleContract {
    /// Validate identity uniqueness, ordered bounds, and a requested class derivative profile.
    pub fn validate(&self, order: DerivativeOrder) -> Result<(), ProblemError> {
        use std::collections::BTreeSet;
        if self.variables.is_empty()
            || self.variables.iter().any(|v| {
                v.lower.is_nan()
                    || v.upper.is_nan()
                    || v.lower > v.upper
                    || v.lower == f64::INFINITY
                    || v.upper == f64::NEG_INFINITY
            })
            || self
                .variables
                .iter()
                .map(|v| v.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.variables.len()
            || self.rows.iter().collect::<BTreeSet<_>>().len() != self.rows.len()
            || self.derivatives < order
            || self.smoothness < order
        {
            return Err(ProblemError::Contract(
                "invalid bounds, duplicate identities or insufficient derivatives/smoothness"
                    .into(),
            ));
        }
        Ok(())
    }
    /// Square root-system admission does not fabricate an optimization objective.
    pub fn square(&self) -> Result<(), ProblemError> {
        self.validate(DerivativeOrder::First)?;
        if self.rows.len() != self.variables.len() {
            return Err(ProblemError::Contract("root system is not square".into()));
        }
        Ok(())
    }
}
/// Square nonlinear equations, with a residual Jacobian or Jacobian-vector product.
pub trait NleOracle: std::fmt::Debug {
    /// Existing compiler matching witness; generic callbacks perform fresh analysis.
    fn structural_analysis(&self) -> Option<&pse_structural::incidence::StructuralAnalysis> {
        None
    }
    /// Admitted original sign guards; absent entries do not imply a sign restriction.
    fn guard_signs(&self) -> std::collections::BTreeMap<SemanticId, pse_math::presolve::GuardSign> {
        Default::default()
    }

    /// Recover authored row observations from independently evaluated residuals.
    /// Generic root oracles declare zero-equality residual functions.
    fn observe(&self, residual: Vec<f64>) -> Result<quality::Observation, ProblemError> {
        let bounds = vec![(0.0, 0.0); residual.len()];
        quality::Observation::from_values(None, residual, bounds)
    }
    /// Exact square-system contract.
    fn contract(&self) -> &OracleContract;
    /// Residual values in declared row order.
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError>;
    /// Canonical assembled residual Jacobian.
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize>;
    /// Values in the canonical sparse Jacobian pattern.
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError>;
    /// Exact residual Jacobian applied to a direction.
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError>;
}
/// Conservatively proved constant-derivative facts for the current numeric assumptions.
/// False means unknown, not a proof of variation.
#[derive(Clone, Copy, Debug, Default)]
pub struct DerivativeFacts {
    /// Every objective gradient component is constant.
    pub gradient_constant: bool,
    /// All original constraint rows are affine.
    pub jacobian_constant: bool,
    /// Affine constraints and constant objective Hessian.
    pub hessian_constant: bool,
}
/// Nonlinear optimization shared only by compatible NLP adapters.
pub trait NlpOracle: std::fmt::Debug {
    /// Existing compiler matching witness for the original equation support.
    fn structural_analysis(&self) -> Option<&pse_structural::incidence::StructuralAnalysis> {
        None
    }
    /// Resolved model coordinates, distinct from optional native algorithmic scaling.
    fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
        None
    }
    /// Original compiled source terms after constraint evaluation; opaque oracles expose none.
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        Ok(vec![])
    }
    /// Source-derived row and objective proofs; absent for opaque callback oracles.
    fn presolve_facts(&self) -> Option<&pse_math::presolve::Facts> {
        None
    }
    /// Proven facts; generic callback oracles conservatively decline.
    fn derivative_facts(&self) -> DerivativeFacts {
        DerivativeFacts::default()
    }
    /// Declared variables and constraint rows.
    fn contract(&self) -> &OracleContract;
    /// Exact sparse constraint Jacobian shape and ordered coefficients.
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize>;
    /// Exact lower-triangle Hessian structure, absent for a first-order profile.
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>>;
    /// Native constraint bounds in declared row order.
    fn constraint_bounds(&self) -> &[(f64, f64)];
    /// Independent callback demand; outputs publish only after successful evaluation.
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError>;
    /// Constraint-only value demand.
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError>;
    /// Objective-only first derivative demand.
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError>;
    /// Values in the canonical sparse Jacobian pattern.
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError>;
    /// Exact lower-triangular Lagrangian Hessian values; absence is explicit in the contract.
    fn hessian(
        &mut self,
        x: &[f64],
        objective_weight: f64,
        multipliers: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError>;
}
/// Validate all derivative dimensions and canonical row bounds before a native NLP upload.
pub fn validate_nlp(oracle: &dyn NlpOracle, order: DerivativeOrder) -> Result<(), ProblemError> {
    oracle.contract().validate(order)?;
    let n = oracle.contract().variables.len();
    let m = oracle.contract().rows.len();
    let j = oracle.jacobian_pattern();
    if j.nrows() != m
        || j.ncols() != n
        || oracle.constraint_bounds().len() != m
        || oracle.constraint_bounds().iter().any(|(l, u)| {
            l.is_nan() || u.is_nan() || l > u || *l == f64::INFINITY || *u == f64::NEG_INFINITY
        })
    {
        return Err(ProblemError::Contract(
            "NLP Jacobian/bound dimensions or values".into(),
        ));
    }
    structural::check(
        oracle.contract(),
        j,
        oracle.constraint_bounds(),
        structural::Mode::Nlp,
        oracle.structural_analysis(),
    )?;
    if order >= DerivativeOrder::Second
        && oracle
            .hessian_pattern()
            .is_none_or(|h| h.nrows() != n || h.ncols() != n)
    {
        return Err(ProblemError::Contract("NLP Hessian dimensions".into()));
    }
    Ok(())
}

/// One objective of a lexicographic coefficient problem (ADR-0111): its linear
/// coefficients and constant in its own sense, and the degradation it admits while later
/// objectives are optimized; the last admits none.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearObjective {
    /// Linear coefficients in column order.
    pub coefficients: Vec<f64>,
    /// Constant term.
    pub constant: f64,
    /// Orientation of this objective.
    pub sense: pse_math::binding::ObjectiveSense,
    /// The degradation this objective admits while later ones are optimized.
    pub degradation: Option<pse_math::binding::Degradation>,
}
/// Library-owned sparse coefficients for linear or quadratic programs.
#[derive(Clone, Debug)]
pub struct CoefficientProblem {
    /// Variable identity and bounds; rows describe linear constraints.
    pub contract: OracleContract,
    /// Linear objective.
    pub objective: Vec<f64>,
    /// Authored objective constant and orientation.
    pub objective_constant: f64,
    /// Authored objective orientation.
    pub sense: pse_math::binding::ObjectiveSense,
    /// One explicit domain per column; a rounded value never implies integrality.
    pub domains: Vec<pse_model::generated::enums::ModelingVariableDomain>,
    /// Fixed/parameter assumptions used during class extraction.
    pub assumptions: ContentHash,
    /// Constraint matrix owned by faer.
    pub constraints: faer::sparse::SparseColMat<usize, f64>,
    /// Optional symmetric objective Hessian, owned by faer.
    pub hessian: Option<faer::sparse::SparseColMat<usize, f64>>,
    /// Row bounds.
    pub bounds: Vec<(f64, f64)>,
    /// Every objective in lexicographic optimization order when the structure has several
    /// (ADR-0111); empty otherwise. The first is `objective`, `objective_constant` and
    /// `sense`.
    pub objectives: Vec<LinearObjective>,
}
impl CoefficientProblem {
    /// Refuse malformed coefficient profiles before a native library sees them.
    pub fn validate(&self) -> Result<(), ProblemError> {
        let last = self.objectives.len().saturating_sub(1);
        if self.objectives.len() == 1
            || self.objectives.iter().enumerate().any(|(k, o)| {
                o.coefficients.len() != self.objective.len()
                    || o.coefficients.iter().any(|v| !v.is_finite())
                    || !o.constant.is_finite()
                    || (k < last) != o.degradation.is_some()
            })
            || self.objectives.first().is_some_and(|o| {
                o.coefficients != self.objective
                    || o.constant.to_bits() != self.objective_constant.to_bits()
                    || o.sense != self.sense
            })
        {
            return Err(ProblemError::Contract(
                "malformed lexicographic objectives".into(),
            ));
        }
        self.contract.validate(DerivativeOrder::Value)?;
        let n = self.contract.variables.len();
        let m = self.contract.rows.len();
        if !self.objective_constant.is_finite()
            || self.domains.len() != n
            || self.objective.len() != n
            || self.objective.iter().any(|x| !x.is_finite())
            || self.constraints.nrows() != m
            || self.constraints.ncols() != n
            || self.constraints.val().iter().any(|x| !x.is_finite())
            || self.bounds.len() != m
            || self.bounds.iter().any(|(l, u)| {
                l.is_nan() || u.is_nan() || l > u || *l == f64::INFINITY || *u == f64::NEG_INFINITY
            })
            || self.hessian.as_ref().is_some_and(|q| {
                q.nrows() != n || q.ncols() != n || q.val().iter().any(|v| !v.is_finite())
            })
        {
            return Err(ProblemError::Contract(
                "malformed coefficient dimensions or values".into(),
            ));
        }
        Ok(())
    }
    /// Exact declared objective convention `constant + c*x + 1/2 x'Qx`.
    pub fn objective_at(&self, x: &[f64]) -> f64 {
        let column = faer::ColRef::from_slice(x);
        let linear = faer::ColRef::from_slice(&self.objective).transpose() * column;
        let quadratic = self.hessian.as_ref().map_or(0.0, |q| {
            let product = q * column;
            0.5 * (column.transpose() * product.as_ref())
        });
        self.objective_constant + linear + quadratic
    }
    // Every coefficient consumer uses the same library product. The runtime's final
    // original symbolic-model evaluation remains independent of solver coefficients.
    fn activity(&self, x: &[f64]) -> Result<Vec<f64>, ProblemError> {
        if x.len() != self.constraints.ncols() || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Internal(
                "coefficient activity dimensions or values".into(),
            ));
        }
        let mut result = vec![0.0; self.constraints.nrows()];
        faer::sparse::linalg::matmul::sparse_dense_matmul(
            faer::MatMut::from_column_major_slice_mut(&mut result, self.constraints.nrows(), 1),
            faer::Accum::Replace,
            self.constraints.as_ref(),
            faer::MatRef::from_column_major_slice(x, x.len(), 1),
            1.0,
            faer::Par::Seq,
        );
        if result.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite coefficient activity"));
        }
        Ok(result)
    }
    /// Recompute all affine rows and disjunctive domain violations independently.
    ///
    /// # Errors
    /// Dimensions, tolerances or candidate values are invalid.
    pub fn quality(
        &self,
        x: &[f64],
        t: &quality::Tolerances,
    ) -> Result<quality::Quality, ProblemError> {
        use pse_model::generated::enums::ModelingVariableDomain;
        use quality::{Violation, interval};
        t.validate(self.contract.variables.len(), self.contract.rows.len())?;
        if x.len() != self.contract.variables.len() || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("invalid coefficient candidate"));
        }
        let activity = self.activity(x)?;
        let rows = self
            .contract
            .rows
            .iter()
            .zip(activity)
            .zip(&self.bounds)
            .zip(&t.rows)
            .map(|(((id, x), (l, u)), t)| Violation {
                id: *id,
                physical: interval(x, *l, *u),
                tolerance: *t,
            })
            .collect();
        let mut bounds = Vec::new();
        let mut integrality = Vec::new();
        for (((v, d), &x), tolerance) in self
            .contract
            .variables
            .iter()
            .zip(&self.domains)
            .zip(x)
            .zip(&t.variables)
        {
            let physical = if d.is_semi() {
                x.abs().min(interval(x, v.lower, v.upper))
            } else {
                interval(
                    x,
                    if *d == ModelingVariableDomain::Binary {
                        v.lower.max(0.0)
                    } else {
                        v.lower
                    },
                    if *d == ModelingVariableDomain::Binary {
                        v.upper.min(1.0)
                    } else {
                        v.upper
                    },
                )
            };
            bounds.push(Violation {
                id: v.id,
                physical,
                tolerance: *tolerance,
            });
            if d.is_integer() {
                integrality.push(Violation {
                    id: v.id,
                    physical: (x - x.round()).abs(),
                    tolerance: t.integrality,
                });
            }
        }
        quality::Quality::new(rows, bounds, integrality)
    }
    /// Independently reconstruct original affine row values and the authored objective.
    /// The native model has shifted row bounds; source constants are applied exactly once.
    ///
    /// # Errors
    /// Dimensions or candidate values are invalid.
    pub fn observation(
        &self,
        x: &[f64],
        constants: &[f64],
        bounds: Vec<(f64, f64)>,
    ) -> Result<quality::Observation, ProblemError> {
        if x.len() != self.contract.variables.len() || constants.len() != self.bounds.len() {
            return Err(ProblemError::Internal(
                "coefficient observation dimensions".into(),
            ));
        }
        let activity = self.activity(x)?;
        let values = activity.iter().zip(constants).map(|(v, c)| v + c).collect();
        quality::Observation::from_values(Some(self.objective_at(x)), values, bounds)
    }
}
/// Explicit conic data in the pse-owned boundary vocabulary ([`conic::SparseMatrix`],
/// [`conic::Cone`]), separate from NLP. Only the Clarabel adapter maps it to native types.
#[derive(Debug)]
pub struct ConicProblem {
    /// Source variable and row identities.
    pub contract: OracleContract,
    /// Symmetric objective matrix, upper triangle only.
    pub quadratic: conic::SparseMatrix,
    /// Linear objective.
    pub objective: Vec<f64>,
    /// Constraint matrix in A x + s = b.
    pub constraints: conic::SparseMatrix,
    /// Right-hand side.
    pub rhs: Vec<f64>,
    /// Cone blocks in constraint row order.
    pub cones: Vec<conic::Cone>,
    /// Constant survives reporting even though native minimization omits it.
    pub objective_constant: f64,
}
impl ConicProblem {
    /// Validate CSC storage, explicit cone parameters, and PSD evidence.
    pub fn validate(
        &self,
        certificate: &dyn pse_math::convexity::QuadraticEvidence,
    ) -> Result<(), ProblemError> {
        use conic::Cone;
        self.contract.validate(DerivativeOrder::Value)?;
        self.quadratic.validate()?;
        self.constraints.validate()?;
        let n = self.contract.variables.len();
        let m = self.contract.rows.len();
        if self.quadratic.rows != n
            || self.quadratic.columns != n
            || self.constraints.rows != m
            || self.constraints.columns != n
            || self.objective.len() != n
            || self.rhs.len() != m
            || !self.objective_constant.is_finite()
            || self
                .objective
                .iter()
                .chain(&self.rhs)
                .chain(&self.quadratic.values)
                .chain(&self.constraints.values)
                .any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract("conic dimensions or values".into()));
        }
        let mut count = 0usize;
        for cone in &self.cones {
            let dim = match cone {
                Cone::Zero { dimension: d }
                | Cone::Nonnegative { dimension: d }
                | Cone::SecondOrder { dimension: d }
                    if *d > 0 =>
                {
                    *d
                }
                Cone::Exponential => 3,
                Cone::PsdTriangle { order: d } if *d > 0 => d
                    .checked_add(1)
                    .and_then(|v| d.checked_mul(v))
                    .and_then(|v| v.checked_div(2))
                    .ok_or_else(|| ProblemError::Unsupported("PSD dimension overflow".into()))?,
                Cone::Power { alpha: a } if a.is_finite() && *a > 0.0 && *a < 1.0 => 3,
                Cone::GeneralizedPower {
                    alpha: a,
                    dimension: d,
                } if *d > 0
                    && !a.is_empty()
                    && a.iter().all(|v| v.is_finite() && *v > 0.0)
                    && (a.iter().sum::<f64>() - 1.0).abs() <= 1e-12 =>
                {
                    a.len().checked_add(*d).ok_or_else(|| {
                        ProblemError::Unsupported("cone dimension overflow".into())
                    })?
                }
                _ => return Err(ProblemError::Contract("invalid explicit cone".into())),
            };
            count = count
                .checked_add(dim)
                .ok_or_else(|| ProblemError::Unsupported("cone dimension overflow".into()))?;
        }
        if count != m {
            return Err(ProblemError::Contract(
                "cone rows differ from constraint inventory".into(),
            ));
        }
        certificate.validate(&self.full_quadratic()?, 1.0)?;
        Ok(())
    }
    /// Symmetric quadratic represented by upper-triangle storage.
    pub fn full_quadratic(&self) -> Result<faer::sparse::SparseColMat<usize, f64>, ProblemError> {
        let n = self.contract.variables.len();
        let mut entries = Vec::with_capacity(self.quadratic.values.len() * 2);
        for c in 0..n {
            for k in self.quadratic.column(c) {
                let r = self.quadratic.row_indices[k];
                let v = self.quadratic.values[k];
                if r > c {
                    return Err(ProblemError::Contract(
                        "conic quadratic must be upper triangular".into(),
                    ));
                }
                entries.push(faer::sparse::Triplet::new(r, c, v));
                if r != c {
                    entries.push(faer::sparse::Triplet::new(c, r, v));
                }
            }
        }
        let q = faer::sparse::SparseColMat::try_new_from_triplets(n, n, &entries)
            .map_err(|e| ProblemError::Internal(e.to_string()))?;
        Ok(q)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn contract() -> OracleContract {
        let id = SemanticId::from_bytes([1; 16]);
        OracleContract {
            identity: ContentHash::from_bytes([1; 32]),
            variables: vec![Variable {
                id,
                lower: 0.0,
                upper: f64::INFINITY,
            }],
            rows: vec![id],
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        }
    }
    #[test]
    fn classes_refuse_invalid_shape_smoothness_and_bounds() {
        let mut value = contract();
        value.square().unwrap();
        assert!(value.validate(DerivativeOrder::Second).is_err());
        value.rows.clear();
        assert!(value.square().is_err());
        value = contract();
        value.smoothness = DerivativeOrder::Value;
        assert!(value.square().is_err());
        value = contract();
        value.variables[0].lower = f64::NAN;
        assert!(value.square().is_err());
    }
    #[test]
    fn malformed_coefficients_and_quality_never_become_an_oracle() {
        let matrix = faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, 1.0)],
        )
        .unwrap();
        let mut p = CoefficientProblem {
            contract: contract(),
            objective: vec![1.0],
            constraints: matrix,
            hessian: None,
            bounds: vec![(0.0, 1.0)],
            objective_constant: 0.0,
            sense: pse_math::binding::ObjectiveSense::Minimize,
            domains: vec![pse_model::generated::enums::ModelingVariableDomain::Continuous],
            assumptions: ContentHash::from_bytes([1; 32]),
            objectives: Vec::new(),
        };
        p.validate().unwrap();
        p.objective.clear();
        assert!(p.validate().is_err());
        assert!(
            quality::Quality::new(
                vec![quality::Violation {
                    id: SemanticId::from_bytes([1; 16]),
                    physical: f64::NAN,
                    tolerance: 1.0
                }],
                vec![],
                vec![]
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod assembled_tests;

#[cfg(test)]
mod solver_tests;

#[cfg(test)]
#[cfg(any(feature = "ipopt", feature = "pounce"))]
mod restart_tests;

#[cfg(test)]
mod clarabel_tests;
#[cfg(test)]
#[cfg(feature = "pounce")]
mod l1_tests;
#[cfg(test)]
mod pounce_convex_tests;

#[cfg(test)]
#[cfg(feature = "scip")]
mod scip_tests;
