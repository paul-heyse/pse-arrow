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
    /// Project frozen ID-keyed budgets into one admitted oracle's coordinate order.
    pub fn from_policy(
        policy: &pse_model::numerics::ResolvedNumericalPolicy,
        variables: &[SemanticId],
        rows: &[SemanticId],
    ) -> Result<Self, ProblemError> {
        use pse_model::generated::enums::NumericalTarget;
        let budget = |ids: &[SemanticId], kind| {
            ids.iter()
                .map(|id| {
                    policy
                        .targets
                        .iter()
                        .find(|t| t.id == *id && t.kind == kind)
                        .map(|t| t.budget)
                        .ok_or_else(|| {
                            ProblemError::Contract(format!("missing resolved budget {id}"))
                        })
                })
                .collect::<Result<Vec<_>, _>>()
        };
        let out = Self {
            variables: budget(variables, NumericalTarget::Variable)?,
            rows: budget(rows, NumericalTarget::Row)?,
            integrality: policy.policy.integrality,
        };
        out.validate(variables.len(), rows.len())?;
        Ok(out)
    }
    /// Comparable normalized feasibility budgets, preserving original physical acceptance.
    pub fn normalized(
        &self,
        n: &pse_math::normalization::Normalization,
    ) -> Result<Self, ProblemError> {
        self.validate(n.variables.len(), n.rows.len())?;
        let scaled = |v: &[f64], s: &[f64]| {
            v.iter()
                .zip(s)
                .map(|(v, s)| pse_math::normalization::checked_ratio(*v, *s))
                .collect::<Result<Vec<_>, _>>()
        };
        Ok(Self {
            variables: scaled(&self.variables, &n.variables)?,
            rows: scaled(&self.rows, &n.rows)?,
            integrality: self.integrality,
        })
    }
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

/// Record achieved KKT quantities in the same normalized meaning as requested accuracy.
/// Missing multipliers remain unavailable; feasibility is an independent assessment.
pub fn record_kkt(
    report: &mut crate::solve::SolveReport,
    n: &pse_math::normalization::Normalization,
    accuracy: &crate::solve::Accuracy,
) {
    use crate::solve::{Metric, Termination};
    let Some(o) = &report.observation else { return };
    let budget = if report.termination.category == Termination::Acceptable {
        accuracy
            .acceptable
            .unwrap_or(pse_model::numerics::KktTolerances {
                stationarity: accuracy.stationarity,
                complementarity: accuracy.complementarity,
            })
    } else {
        pse_model::numerics::KktTolerances {
            stationarity: accuracy.stationarity,
            complementarity: accuracy.complementarity,
        }
    };
    let stationarity = o.stationarity.as_ref().map(|values| {
        if values.len() != n.variables.len() {
            return Err(ProblemError::Contract(
                "stationarity coordinate extent".into(),
            ));
        }
        values
            .iter()
            .zip(&n.variables)
            .try_fold(0.0_f64, |m, (v, s)| {
                Ok(m.max(pse_math::normalization::checked_ratio(
                    pse_math::normalization::checked_product(v.abs(), *s)?,
                    n.objective,
                )?))
            })
    });
    let complementarity = o.complementarity.as_ref().map(|values| {
        values.iter().try_fold(0.0_f64, |m, v| {
            Ok::<_, ProblemError>(m.max(pse_math::normalization::checked_ratio(
                v.abs(),
                n.objective,
            )?))
        })
    });
    for (name, value, budget) in [
        ("stationarity", stationarity, budget.stationarity),
        ("complementarity", complementarity, budget.complementarity),
    ] {
        report
            .metrics
            .insert(format!("quality.{name}.budget"), Metric::Real(budget));
        match value {
            Some(Ok(v)) => {
                report
                    .metrics
                    .insert(format!("quality.{name}.normalized"), Metric::Real(v));
                report.metrics.insert(
                    format!("quality.{name}.accepted"),
                    Metric::Bool(v <= budget),
                );
            }
            Some(Err(e)) => {
                report.metrics.insert(
                    format!("quality.{name}.unavailable"),
                    Metric::Text(e.to_string()),
                );
            }
            None => {
                report.metrics.insert(
                    format!("quality.{name}.unavailable"),
                    Metric::Text("original derivative or multipliers unavailable".into()),
                );
            }
        }
    }
}

/// Grant only the numerical claim supported by completed original-space observations.
/// Native stop categories are retained independently, including limits with feasible candidates.
pub fn qualify(report: &mut crate::solve::SolveReport, accuracy: &crate::solve::Accuracy) {
    use crate::solve::{Assurance, Backend, Metric, Qualification, Termination};
    report.qualification = Qualification::Unqualified;
    report.termination.assurance = Assurance::None;
    if report.validation_error.is_some()
        || report.candidate.is_none()
        || !report.quality.as_ref().is_some_and(Quality::feasible)
    {
        return;
    }
    report.qualification = Qualification::Feasible;
    report.termination.assurance = Assurance::Feasible;
    let accepted = |key: &str| matches!(report.metrics.get(key), Some(Metric::Bool(true)));
    let real = |key: &str| match report.metrics.get(key) {
        Some(Metric::Real(v)) if v.is_finite() => Some(*v),
        _ => None,
    };
    let stopped = matches!(
        report.termination.category,
        Termination::Success | Termination::Acceptable
    );
    if !stopped {
        return;
    }
    match report.backend {
        Backend::Ipopt | Backend::Pounce
            if report
                .observation
                .as_ref()
                .is_some_and(|o| o.dual_error.is_none())
                && accepted("quality.stationarity.accepted")
                && accepted("quality.complementarity.accepted")
                && (report.termination.category != Termination::Acceptable
                    || accuracy.acceptable.is_some()) =>
        {
            report.qualification = Qualification::Stationary;
            report.termination.assurance = Assurance::LocalStationary;
        }
        Backend::Highs if accepted("upload.equivalent") => {
            if accepted("model.discrete") {
                let (Some(gap), Some(bound)) = (real("mip_gap"), real("mip_dual_bound")) else {
                    return;
                };
                let value = real("objective_function_value");
                let absolute = value.map(|v| (v - bound).abs());
                if gap >= 0.0
                    && (gap <= accuracy.mip_relative_gap
                        || absolute.is_some_and(|v| v <= accuracy.mip_absolute_gap))
                {
                    report.qualification = if gap == 0.0 && absolute == Some(0.0) {
                        Qualification::OptimalWithinTolerance
                    } else {
                        Qualification::GapQualified
                    };
                    report.termination.assurance = Assurance::NativeOptimal;
                }
            } else if real("max_dual_infeasibility")
                .is_some_and(|v| v >= 0.0 && v <= accuracy.stationarity)
                && real("primal_dual_objective_error")
                    .is_some_and(|v| v >= 0.0 && v <= accuracy.gap_relative)
                && matches!(
                    report.metrics.get("dual_solution_status"),
                    Some(Metric::Integer(2))
                )
            {
                report.qualification = Qualification::OptimalWithinTolerance;
                report.termination.assurance = Assurance::NativeOptimal;
            }
        }
        Backend::Clarabel
            if real("res_primal").is_some_and(|v| v <= accuracy.feasibility)
                && real("res_dual").is_some_and(|v| v <= accuracy.stationarity)
                && (real("gap_abs").is_some_and(|v| v <= accuracy.gap_absolute)
                    || real("gap_rel").is_some_and(|v| v <= accuracy.gap_relative)) =>
        {
            report.qualification = Qualification::OptimalWithinTolerance;
            report.termination.assurance = Assurance::NativeOptimal;
        }
        _ => {}
    }
}

#[cfg(test)]
mod qualification_tests {
    use super::*;
    use crate::{OracleContract, Variable, solve::*};
    fn report(backend: Backend, category: Termination) -> SolveReport {
        let contract = OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![Variable {
                id: SemanticId::from_bytes([1; 16]),
                lower: f64::NEG_INFINITY,
                upper: f64::INFINITY,
            }],
            rows: vec![],
            derivatives: pse_kernels::DerivativeOrder::Second,
            smoothness: pse_kernels::DerivativeOrder::Second,
        };
        let execution = Execution::new(
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &Controls::default(),
        );
        let mut r = SolveReport::new(
            backend,
            &contract,
            NativeTermination {
                code: 0,
                name: "fixture".into(),
                message: None,
                category,
                assurance: Assurance::NativeOptimal,
            },
            &execution,
        );
        r.candidate = Some(Candidate {
            kind: CandidateKind::FinalIterate,
            primal: vec![0.0],
            objective: Some(10.0),
            row_dual: None,
            bound_dual: None,
            reduced_costs: None,
            slacks: None,
        });
        r.quality = Some(Quality::new(vec![], vec![], vec![]).unwrap());
        r
    }
    #[test]
    fn acceptable_kkt_requires_original_evidence_and_explicit_policy() {
        let mut r = report(Backend::Ipopt, Termination::Acceptable);
        let mut o = Observation::from_values(Some(10.0), vec![], vec![]).unwrap();
        o.dual_error = None;
        o.stationarity = Some(vec![1e-4]);
        o.complementarity = Some(vec![1e-4]);
        r.observation = Some(o);
        let n = pse_math::normalization::Normalization::identity(1, 0);
        let mut accuracy = Accuracy::default();
        record_kkt(&mut r, &n, &accuracy);
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Feasible);
        accuracy.acceptable = Some(pse_model::numerics::KktTolerances {
            stationarity: 1e-3,
            complementarity: 1e-3,
        });
        record_kkt(&mut r, &n, &accuracy);
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Stationary);
        r.validation_error = Some("original callback failed".into());
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Unqualified);
    }
    #[test]
    fn gaps_upload_evidence_and_limits_have_separate_meanings() {
        let mut r = report(Backend::Highs, Termination::Success);
        r.metrics.extend([
            ("model.discrete".into(), Metric::Bool(true)),
            ("mip_gap".into(), Metric::Real(0.02)),
            ("mip_dual_bound".into(), Metric::Real(9.8)),
            ("objective_function_value".into(), Metric::Real(10.0)),
        ]);
        let accuracy = Accuracy {
            mip_relative_gap: 0.03,
            ..Default::default()
        };
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Feasible);
        r.metrics
            .insert("upload.equivalent".into(), Metric::Bool(true));
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::GapQualified);
        for category in [
            Termination::IterationLimit,
            Termination::ResourceExhausted,
            Termination::Inconclusive,
        ] {
            r.termination.category = category;
            qualify(&mut r, &accuracy);
            assert_eq!(r.qualification, Qualification::Feasible);
            assert_eq!(r.termination.category, category);
        }
        r.termination.category = Termination::Success;
        r.metrics.insert("mip_gap".into(), Metric::Real(f64::NAN));
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Feasible);
    }
}
