// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-space quality retains row identity instead of combining incompatible units.
use crate::{NlpOracle, ProblemError};
use pse_ids::SemanticId;
/// A violation in one declared physical representation.
#[derive(Clone, Debug)]
pub struct Violation {
    /// Original row or variable.
    pub id: SemanticId,
    /// Nonnegative physical violation in that source's units.
    pub physical: f64,
    /// Positive explicit physical tolerance in the same units.
    pub tolerance: f64,
}
/// Explicit physical acceptance scales; never inferred from a large trial value.
#[derive(Clone, Debug)]
pub struct Tolerances {
    /// Per-variable bound tolerance.
    pub variables: Vec<f64>,
    /// Per-row residual/constraint tolerance.
    pub rows: Vec<f64>,
    /// Integrality tolerance, dimensionless.
    pub integrality: f64,
}
impl Tolerances {
    /// Check exact inventories and strictly positive finite tolerances.
    pub fn validate(&self, n: usize, m: usize) -> Result<(), ProblemError> {
        if self.variables.len() != n
            || self.rows.len() != m
            || self
                .variables
                .iter()
                .chain(&self.rows)
                .chain(std::iter::once(&self.integrality))
                .any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err(ProblemError::Contract(
                "physical quality tolerance dimensions/values".into(),
            ));
        }
        Ok(())
    }
}
/// Original mathematical feasibility in declared physical units; conservation is assessed separately.
/// Maxima are dimensionless ratios only.
#[derive(Clone, Debug)]
pub struct Quality {
    /// Original constraint violations by source.
    pub rows: Vec<Violation>,
    /// Original variable bound violations by source.
    pub bounds: Vec<Violation>,
    /// Discrete domain violations, dimensionless.
    pub integrality: Vec<Violation>,
    /// Maximum violation divided by its explicit tolerance.
    pub normalized_max: f64,
}
/// Values independently evaluated in the original declared coordinates.
#[derive(Clone, Debug)]
pub struct Observation {
    /// Original source outputs before equation aggregation, when exposed by the compiled oracle.
    pub sources: Vec<pse_math::assembly::OutputValue>,
    /// Re-evaluated authored objective, absent for a root problem.
    pub objective: Option<f64>,
    /// Constraint values, not residuals.
    pub values: Vec<f64>,
    /// Original lower and upper row bounds.
    pub bounds: Vec<(f64, f64)>,
    /// Signed residual for finite equality rows only.
    pub equality_residuals: Vec<Option<f64>>,
    /// Per-side physical violations.
    pub lower_violations: Vec<f64>,
    /// Per-side physical violations.
    pub upper_violations: Vec<f64>,
    /// Original-coordinate stationarity, in the documented minimization convention.
    pub stationarity: Option<Vec<f64>>,
    /// Nonnegative magnitude of bound/row complementarity products.
    pub complementarity: Option<Vec<f64>>,
    /// Missing/invalid multipliers or derivative evaluation error.
    pub dual_error: Option<String>,
}
impl Observation {
    /// Retain raw values and compute residuals only where their meaning is declared.
    pub fn from_values(
        objective: Option<f64>,
        values: Vec<f64>,
        bounds: Vec<(f64, f64)>,
    ) -> Result<Self, ProblemError> {
        if values.len() != bounds.len()
            || values.iter().any(|v| !v.is_finite())
            || objective.is_some_and(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "original observation dimensions/values".into(),
            ));
        }
        Ok(Self {
            sources: vec![],
            objective,
            equality_residuals: values
                .iter()
                .zip(&bounds)
                .map(|(v, (l, u))| (l == u && l.is_finite()).then_some(v - l))
                .collect(),
            lower_violations: values
                .iter()
                .zip(&bounds)
                .map(|(v, (l, _))| (l - v).max(0.0))
                .collect(),
            upper_violations: values
                .iter()
                .zip(&bounds)
                .map(|(v, (_, u))| (v - u).max(0.0))
                .collect(),
            values,
            bounds,
            stationarity: None,
            complementarity: None,
            dual_error: Some("multipliers unavailable".into()),
        })
    }
}
/// Final observation never substitutes the native solver's possibly stale row buffer.
pub fn attach_nlp(
    report: &mut crate::solve::SolveReport,
    oracle: &mut dyn NlpOracle,
    tolerance: &Tolerances,
    sense: pse_math::binding::ObjectiveSense,
) {
    let Some(candidate) = report.candidate.as_ref() else {
        return;
    };
    let result = contained(|| {
        let mut values = vec![0.0; oracle.contract().rows.len()];
        oracle.constraints(&candidate.primal, &mut values)?;
        let quality = observed(
            oracle.contract(),
            oracle.constraint_bounds(),
            &candidate.primal,
            &values,
            tolerance,
        )?;
        let objective = oracle.objective(&candidate.primal)? * sense.sign();
        let mut observation =
            Observation::from_values(Some(objective), values, oracle.constraint_bounds().to_vec())?;
        observation.sources = oracle.constraint_sources()?;
        match kkt(oracle, candidate, &observation) {
            Ok((s, c)) => {
                observation.stationarity = Some(s);
                observation.complementarity = Some(c);
                observation.dual_error = None;
            }
            Err(e) => observation.dual_error = Some(e.to_string()),
        }
        Ok((quality, observation))
    });
    match result {
        Ok((q, o)) => {
            if !q.feasible() {
                report.termination.assurance = crate::solve::Assurance::None;
            }
            report.quality = Some(q);
            report.observation = Some(o);
            report.validation_error = None;
        }
        Err(e) => {
            report.quality = None;
            report.observation = None;
            report.validation_error = Some(e.to_string());
            report.termination.assurance = crate::solve::Assurance::None;
        }
    }
}
fn kkt(
    oracle: &mut dyn NlpOracle,
    c: &crate::solve::Candidate,
    o: &Observation,
) -> Result<(Vec<f64>, Vec<f64>), ProblemError> {
    let n = oracle.contract().variables.len();
    let m = oracle.contract().rows.len();
    let lambda = c
        .row_dual
        .as_ref()
        .ok_or_else(|| ProblemError::Contract("row multipliers unavailable".into()))?;
    let (zl, zu) = c
        .bound_dual
        .as_ref()
        .ok_or_else(|| ProblemError::Contract("bound multipliers unavailable".into()))?;
    if lambda.len() != m
        || zl.len() != n
        || zu.len() != n
        || lambda.iter().chain(zl).chain(zu).any(|v| !v.is_finite())
        || zl.iter().chain(zu).any(|v| *v < 0.0)
    {
        return Err(ProblemError::Contract("dual dimensions/values/sign".into()));
    }
    let mut gradient = vec![0.0; n];
    oracle.gradient(&c.primal, &mut gradient)?;
    let mut jac = vec![0.0; oracle.jacobian_pattern().row_idx().len()];
    oracle.jacobian(&c.primal, &mut jac)?;
    let pattern = oracle.jacobian_pattern();
    let mut comp = Vec::with_capacity(2 * n + m);
    for col in 0..n {
        for k in pattern.col_ptr()[col]..pattern.col_ptr()[col + 1] {
            gradient[col] += jac[k] * lambda[pattern.row_idx()[k]];
        }
        gradient[col] += -zl[col] + zu[col];
        let v = &oracle.contract().variables[col];
        if v.lower.is_finite() {
            comp.push((zl[col] * (c.primal[col] - v.lower)).abs());
        } else if zl[col] != 0.0 {
            return Err(ProblemError::Contract(
                "dual on absent lower variable bound".into(),
            ));
        }
        if v.upper.is_finite() {
            comp.push((zu[col] * (v.upper - c.primal[col])).abs());
        } else if zu[col] != 0.0 {
            return Err(ProblemError::Contract(
                "dual on absent upper variable bound".into(),
            ));
        }
    }
    for (r, lambda) in lambda.iter().enumerate() {
        let (l, u) = o.bounds[r];
        if l != u {
            let bound = if *lambda < 0.0 { l } else { u };
            if bound.is_finite() {
                comp.push((lambda * (o.values[r] - bound)).abs());
            } else if *lambda != 0.0 {
                return Err(ProblemError::Contract("dual on absent row bound".into()));
            }
        }
    }
    if gradient.iter().chain(&comp).any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract("nonfinite original KKT".into()));
    }
    Ok((gradient, comp))
}
impl Quality {
    /// Whether independently recomputed violations meet every supplied tolerance.
    pub fn feasible(&self) -> bool {
        self.normalized_max <= 1.0
    }
    /// Compute dimensionless aggregate while retaining every physical measurement.
    pub fn new(
        rows: Vec<Violation>,
        bounds: Vec<Violation>,
        integrality: Vec<Violation>,
    ) -> Result<Self, ProblemError> {
        let mut normalized_max: f64 = 0.0;
        for v in rows.iter().chain(&bounds).chain(&integrality) {
            if !v.physical.is_finite()
                || v.physical < 0.0
                || !v.tolerance.is_finite()
                || v.tolerance <= 0.0
            {
                return Err(ProblemError::Contract(
                    "invalid original quality measurement".into(),
                ));
            }
            normalized_max = normalized_max.max(v.physical / v.tolerance);
        }
        Ok(Self {
            rows,
            bounds,
            integrality,
            normalized_max,
        })
    }
}
/// Preserve the completed native report if a user/provider validator unwinds.
pub(crate) fn contained<T>(
    work: impl FnOnce() -> Result<T, ProblemError>,
) -> Result<T, ProblemError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).unwrap_or_else(|_| {
        Err(ProblemError::Contract(
            "panic during original-model validation".into(),
        ))
    })
}
/// Closed-interval violation, including outward infinite bounds.
pub fn interval(x: f64, lower: f64, upper: f64) -> f64 {
    if !x.is_finite() {
        return f64::INFINITY;
    }
    (lower - x).max(x - upper).max(0.0)
}
/// Recompute constraints through the original oracle after native execution.
pub fn nlp(
    oracle: &mut dyn NlpOracle,
    x: &[f64],
    tolerance: &Tolerances,
) -> Result<Quality, ProblemError> {
    let mut values = vec![0.0; oracle.contract().rows.len()];
    oracle.constraints(x, &mut values)?;
    observed(
        oracle.contract(),
        oracle.constraint_bounds(),
        x,
        &values,
        tolerance,
    )
}
/// Quality from one independently observed value buffer, reused by all result adapters.
pub fn observed(
    contract: &crate::OracleContract,
    limits: &[(f64, f64)],
    x: &[f64],
    values: &[f64],
    tolerance: &Tolerances,
) -> Result<Quality, ProblemError> {
    let n = contract.variables.len();
    let m = contract.rows.len();
    tolerance.validate(n, m)?;
    if x.len() != n || values.len() != m || limits.len() != m || x.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract("nonfinite candidate".into()));
    }
    let bounds = contract
        .variables
        .iter()
        .zip(x)
        .zip(&tolerance.variables)
        .map(|((v, x), t)| Violation {
            id: v.id,
            physical: interval(*x, v.lower, v.upper),
            tolerance: *t,
        })
        .collect();
    let ids = contract.rows.clone();
    if values.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract("nonfinite original residual".into()));
    }
    let rows = ids
        .iter()
        .zip(values)
        .zip(limits)
        .zip(&tolerance.rows)
        .map(|(((id, v), (l, u)), t)| Violation {
            id: *id,
            physical: interval(*v, *l, *u),
            tolerance: *t,
        })
        .collect();
    Quality::new(rows, bounds, vec![])
}
