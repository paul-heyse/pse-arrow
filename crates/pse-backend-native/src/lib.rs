// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Class-specific native problem contracts. Execution adapters are supplied by Plan 14 M11–M17.
pub mod assembled;
pub mod callback;
pub mod conic;
mod convexity;
pub mod dynamics;
#[cfg(feature = "highs")]
pub mod highs;
#[cfg(feature = "ipopt")]
pub mod ipopt;
#[cfg(feature = "kinsol")]
pub mod kinsol;
mod nlp_pattern;
#[cfg(feature = "pounce")]
pub mod pounce;
pub mod presolve;
pub mod quality;
#[cfg(feature = "kinsol")]
pub mod recycle;
pub mod routing;
pub mod solve;
/// Original-equation structural admission.
pub mod structural;
pub mod tears;
mod tnlp;
pub use convexity::GramCertificate;
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::DerivativeOrder;
#[cfg(test)]
use std::sync::{Arc, atomic::AtomicBool};

/// A malformed problem or an attributable trial failure.
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
    /// Contract admission failed before entering a solver.
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
}
pse_diagnostics::impl_diagnostic! {
    ProblemError,
    code(this) { match this { Self::Contract(_) | Self::Unavailable{..} | Self::Structural{..} => Some(pse_diagnostics::DiagnosticCode::CompileMath), Self::Math(_) => None } },
    forward(this) { match this { Self::Math(error) => Some(error), _ => None } },
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
    /// Original compiled source terms after constraint evaluation; opaque oracles expose none.
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        Ok(vec![])
    }
    /// Source-derived row and objective proofs; absent for opaque callback oracles.
    fn presolve_facts(&self) -> Option<&pse_math::presolve::Facts> {
        None
    }
    /// Native variable scaling, independent of physical unit conversion.
    fn scaling(&self) -> Option<&presolve::Scaling> {
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
    structural::oracle(
        oracle.contract(),
        j,
        oracle.constraint_bounds(),
        structural::Mode::Nlp,
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

/// Library-owned sparse coefficients for linear or quadratic programs.
#[derive(Debug)]
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
    pub domains: Vec<pse_math::binding::VariableDomain>,
    /// Fixed/parameter assumptions used during class extraction.
    pub assumptions: ContentHash,
    /// Constraint matrix owned by faer.
    pub constraints: faer::sparse::SparseColMat<usize, f64>,
    /// Optional symmetric objective Hessian, owned by faer.
    pub hessian: Option<faer::sparse::SparseColMat<usize, f64>>,
    /// Row bounds.
    pub bounds: Vec<(f64, f64)>,
}
impl CoefficientProblem {
    /// Refuse malformed coefficient profiles before a native library sees them.
    pub fn validate(&self) -> Result<(), ProblemError> {
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
}
/// Native conic data in Clarabel's own matrix and cone vocabulary, separate from NLP.
#[derive(Debug)]
pub struct ConicProblem {
    /// Source variable and row identities.
    pub contract: OracleContract,
    /// Symmetric objective matrix, upper triangle only.
    pub quadratic: clarabel::algebra::CscMatrix<f64>,
    /// Linear objective.
    pub objective: Vec<f64>,
    /// Constraint matrix in A x + s = b.
    pub constraints: clarabel::algebra::CscMatrix<f64>,
    /// Right-hand side.
    pub rhs: Vec<f64>,
    /// Cone blocks in constraint row order.
    pub cones: Vec<clarabel::solver::SupportedConeT<f64>>,
    /// Constant survives reporting even though native minimization omits it.
    pub objective_constant: f64,
}
impl ConicProblem {
    /// Validate library CSC storage, explicit cone parameters, and PSD evidence.
    pub fn validate(&self, certificate: &GramCertificate) -> Result<(), ProblemError> {
        use clarabel::solver::SupportedConeT::{
            ExponentialConeT, GenPowerConeT, NonnegativeConeT, PowerConeT, SecondOrderConeT,
            ZeroConeT,
        };
        self.contract.validate(DerivativeOrder::Value)?;
        self.quadratic
            .check_format()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        self.constraints
            .check_format()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let n = self.contract.variables.len();
        let m = self.contract.rows.len();
        if self.quadratic.m != n
            || self.quadratic.n != n
            || self.constraints.m != m
            || self.constraints.n != n
            || self.objective.len() != n
            || self.rhs.len() != m
            || !self.objective_constant.is_finite()
            || self
                .objective
                .iter()
                .chain(&self.rhs)
                .chain(&self.quadratic.nzval)
                .chain(&self.constraints.nzval)
                .any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract("conic dimensions or values".into()));
        }
        let mut count = 0usize;
        for cone in &self.cones {
            let dim = match cone {
                ZeroConeT(d) | NonnegativeConeT(d) | SecondOrderConeT(d) if *d > 0 => *d,
                ExponentialConeT() => 3,
                #[cfg(feature = "sdp")]
                clarabel::solver::SupportedConeT::PSDTriangleConeT(d) if *d > 0 => d
                    .checked_add(1)
                    .and_then(|v| d.checked_mul(v))
                    .and_then(|v| v.checked_div(2))
                    .ok_or_else(|| ProblemError::Contract("PSD dimension overflow".into()))?,
                PowerConeT(a) if a.is_finite() && *a > 0.0 && *a < 1.0 => 3,
                GenPowerConeT(a, d)
                    if *d > 0
                        && !a.is_empty()
                        && a.iter().all(|v| v.is_finite() && *v > 0.0)
                        && (a.iter().sum::<f64>() - 1.0).abs() <= 1e-12 =>
                {
                    a.len()
                        .checked_add(*d)
                        .ok_or_else(|| ProblemError::Contract("cone dimension overflow".into()))?
                }
                _ => return Err(ProblemError::Contract("invalid explicit cone".into())),
            };
            count = count
                .checked_add(dim)
                .ok_or_else(|| ProblemError::Contract("cone dimension overflow".into()))?;
        }
        if count != m {
            return Err(ProblemError::Contract(
                "cone rows differ from constraint inventory".into(),
            ));
        }
        let mut entries = Vec::with_capacity(self.quadratic.nzval.len() * 2);
        for c in 0..n {
            for k in self.quadratic.colptr[c]..self.quadratic.colptr[c + 1] {
                let r = self.quadratic.rowval[k];
                let v = self.quadratic.nzval[k];
                if r > c {
                    return Err(ProblemError::Contract(
                        "Clarabel quadratic must be upper triangular".into(),
                    ));
                }
                entries.push(faer::sparse::Triplet::new(r, c, v));
                if r != c {
                    entries.push(faer::sparse::Triplet::new(c, r, v));
                }
            }
        }
        let q = faer::sparse::SparseColMat::try_new_from_triplets(n, n, &entries)
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        certificate.validate(&q, 1.0)?;
        Ok(())
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
            domains: vec![pse_math::binding::VariableDomain::Continuous],
            assumptions: ContentHash::from_bytes([1; 32]),
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
