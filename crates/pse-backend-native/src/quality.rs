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
        if values.len() != bounds.len() {
            return Err(ProblemError::internal("original observation dimensions"));
        }
        if values.iter().any(|v| !v.is_finite()) || objective.is_some_and(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite original observation"));
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
            report.clear_validation_failure();
        }
        Err(e) => {
            report.quality = None;
            report.observation = None;
            report.record_validation_failure(e);
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
        .ok_or_else(|| ProblemError::unsupported("row multipliers unavailable"))?;
    let (zl, zu) = c
        .bound_dual
        .as_ref()
        .ok_or_else(|| ProblemError::unsupported("bound multipliers unavailable"))?;
    if lambda.len() != m
        || zl.len() != n
        || zu.len() != n
        || lambda.iter().chain(zl).chain(zu).any(|v| !v.is_finite())
        || zl.iter().chain(zu).any(|v| *v < 0.0)
    {
        return Err(ProblemError::numerical("dual dimensions/values/sign"));
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
            return Err(ProblemError::numerical(
                "dual on absent lower variable bound",
            ));
        }
        if v.upper.is_finite() {
            comp.push((zu[col] * (v.upper - c.primal[col])).abs());
        } else if zu[col] != 0.0 {
            return Err(ProblemError::numerical(
                "dual on absent upper variable bound",
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
                return Err(ProblemError::numerical("dual on absent row bound"));
            }
        }
    }
    if gradient.iter().chain(&comp).any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite original KKT"));
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
                return Err(ProblemError::numerical(
                    "invalid original quality measurement",
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
        Err(ProblemError::internal(
            "panic during original-model validation",
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
    if x.len() != n || values.len() != m || limits.len() != m {
        return Err(ProblemError::internal("candidate observation dimensions"));
    }
    if x.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite candidate"));
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
        return Err(ProblemError::numerical("nonfinite original residual"));
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
/// The typed acceptance is evidence for qualification; the metrics are observations.
pub fn record_kkt(
    report: &mut crate::solve::SolveReport,
    n: &pse_math::normalization::Normalization,
    accuracy: &crate::solve::ResolvedAccuracy,
) {
    use crate::solve::{KktEvidence, Metric, Termination};
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
            return Err(ProblemError::internal("stationarity coordinate extent"));
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
    let mut evidence = KktEvidence::default();
    for (name, value, budget, accepted) in [
        (
            "stationarity",
            stationarity,
            budget.stationarity,
            &mut evidence.stationarity,
        ),
        (
            "complementarity",
            complementarity,
            budget.complementarity,
            &mut evidence.complementarity,
        ),
    ] {
        report
            .metrics
            .insert(format!("quality.{name}.budget"), Metric::Real(budget));
        match value {
            Some(Ok(v)) => {
                *accepted = Some(v <= budget);
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
    report.evidence.kkt = Some(evidence);
}

/// The least-infeasible label of a report whose native stop is a local infeasibility with a
/// candidate (ADR-0109 item 3): the original rows the candidate leaves violated beyond their
/// budgets. `None` for every other stop.
pub fn least_infeasible(
    report: &crate::solve::SolveReport,
) -> Option<crate::solve::LeastInfeasible> {
    (report.termination.category == crate::solve::Termination::Infeasible
        && report.candidate.is_some())
    .then(|| crate::solve::LeastInfeasible {
        violated: report
            .quality
            .as_ref()
            .map(|q| {
                q.rows
                    .iter()
                    .filter(|v| v.physical > v.tolerance)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default(),
    })
}
/// Grant only the numerical claim supported by completed original-space observations.
/// Native stop categories are retained independently, including limits with feasible candidates.
/// Only typed adapter evidence is read; metrics never grant a claim.
pub fn qualify(report: &mut crate::solve::SolveReport, accuracy: &crate::solve::ResolvedAccuracy) {
    use crate::solve::{Assurance, Qualification, SolutionStatus, Termination};
    report.qualification = Qualification::Unqualified;
    report.termination.assurance = Assurance::None;
    if let Some(global) = report.evidence.global {
        qualify_global(report, global);
        return;
    }
    if report.validation_failure().is_some()
        || report.candidate.is_none()
        || !report.quality.as_ref().is_some_and(Quality::feasible)
    {
        return;
    }
    report.qualification = Qualification::Feasible;
    report.termination.assurance = Assurance::Feasible;
    let finite = |v: Option<f64>| v.filter(|v| v.is_finite());
    let stopped = matches!(
        report.termination.category,
        Termination::Success | Termination::Acceptable
    );
    if !stopped {
        return;
    }
    // The typed evidence an adapter records selects the class rule; no backend is named,
    // so a new adapter qualifies through the evidence it produces.
    let (coefficient, conic, kkt) = (
        report.evidence.coefficient,
        report.evidence.conic,
        report.evidence.kkt,
    );
    if let Some(c) = coefficient {
        if !c.upload_equivalent {
            return;
        }
        if c.discrete {
            let (Some(gap), Some(bound)) = (finite(c.mip_gap), finite(c.mip_dual_bound)) else {
                return;
            };
            let absolute = finite(c.objective).map(|v| (v - bound).abs());
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
        } else if finite(c.max_dual_infeasibility)
            .is_some_and(|v| v >= 0.0 && v <= accuracy.stationarity)
            && finite(c.primal_dual_objective_error)
                .is_some_and(|v| v >= 0.0 && v <= accuracy.gap_relative)
            && c.dual == SolutionStatus::Feasible
        {
            report.qualification = Qualification::OptimalWithinTolerance;
            report.termination.assurance = Assurance::NativeOptimal;
        }
    } else if let Some(c) = conic {
        if c.primal_residual <= accuracy.feasibility
            && c.dual_residual <= accuracy.stationarity
            && (c.gap_absolute <= accuracy.gap_absolute || c.gap_relative <= accuracy.gap_relative)
        {
            report.qualification = Qualification::OptimalWithinTolerance;
            report.termination.assurance = Assurance::NativeOptimal;
        }
    } else if report
        .observation
        .as_ref()
        .is_some_and(|o| o.dual_error.is_none())
        && kkt.is_some_and(|k| k.stationarity == Some(true) && k.complementarity == Some(true))
        && (report.termination.category != Termination::Acceptable || accuracy.acceptable.is_some())
    {
        report.qualification = Qualification::Stationary;
        report.termination.assurance = Assurance::LocalStationary;
    }
}

/// Global evidence (ADR-0105 §2, ADR-0106 §9–§10). Bound and infeasibility claims need a
/// readback-equivalent export and a successful or infeasible stop; they hold within the
/// recorded tolerances and export fidelity. A gap claim additionally needs an
/// original-feasible candidate from a result source, whose fresh original objective lies
/// within the recorded gap of the dual bound. A relaxed incumbent never becomes a
/// solution claim. A conclusion reached in rational arithmetic over an exact export is an
/// exact certificate, the only rigorous assurance (ADR-0106 §9).
fn qualify_global(report: &mut crate::solve::SolveReport, g: crate::solve::GlobalEvidence) {
    use crate::solve::{Assurance, BoundSource, PrimalSource, Qualification, Termination};
    let feasible = report.validation_failure().is_none()
        && report.candidate.is_some()
        && report.quality.as_ref().is_some_and(Quality::feasible);
    if feasible {
        report.qualification = Qualification::Feasible;
        report.termination.assurance = Assurance::Feasible;
    }
    if !g.readback {
        return;
    }
    let exact = g.exact && g.dual == BoundSource::ExactExport;
    match report.termination.category {
        Termination::Infeasible if g.infeasible => {
            report.termination.assurance = if exact {
                Assurance::ExactCertificate
            } else {
                Assurance::ProvenInfeasible
            };
            return;
        }
        Termination::Success => {}
        _ => return,
    }
    let Some(bound) = g.dual_bound.filter(|v| v.is_finite()) else {
        return;
    };
    report.termination.assurance = if exact {
        Assurance::ExactCertificate
    } else {
        Assurance::GlobalBound
    };
    if !feasible || g.primal == PrimalSource::RelaxedIncumbent {
        return;
    }
    let Some(objective) = report
        .observation
        .as_ref()
        .and_then(|o| o.objective)
        .filter(|v| v.is_finite())
    else {
        return;
    };
    // The backend's gap: absolute, or relative to the smaller magnitude when both share
    // a sign (a sign change makes the relative gap infinite).
    let absolute = (objective - bound).abs();
    let relative = objective.signum() == bound.signum()
        && absolute <= g.gap_relative * objective.abs().min(bound.abs());
    if absolute <= g.gap_absolute || relative {
        // An exact proof closes the gap in rational arithmetic; the candidate's objective
        // is still the original-coordinate evaluation.
        report.qualification = if exact {
            Qualification::OptimalWithinTolerance
        } else {
            Qualification::GapQualified
        };
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
        let mut accuracy = ResolvedAccuracy::nominal();
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
        r.record_validation_failure(ProblemError::numerical("original callback failed"));
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Unqualified);
    }
    #[test]
    fn original_validation_keeps_typed_failure_until_a_fresh_observation_succeeds() {
        let mut r = report(Backend::Ipopt, Termination::Success);
        let source_id = SemanticId::from_bytes([17; 16]);
        r.record_validation_failure(
            pse_math::MathError::Domain {
                source_id,
                requirement: "positive",
            }
            .into(),
        );
        qualify(&mut r, &ResolvedAccuracy::nominal());
        assert_eq!(r.qualification, Qualification::Unqualified);
        let retained = r.clone();
        r.clear_validation_failure();
        assert!(r.validation_failure().is_none());
        assert!(
            matches!(retained.validation_failure(),Some(ProblemError::Math(pse_math::MathError::Domain {source_id:id,..})) if *id==source_id)
        );
        assert!(retained.failure_bytes() > 0);
    }
    fn coefficient() -> CoefficientEvidence {
        CoefficientEvidence {
            upload_equivalent: true,
            discrete: true,
            objective: Some(10.0),
            mip_gap: Some(0.02),
            mip_dual_bound: Some(9.8),
            primal: SolutionStatus::Feasible,
            dual: SolutionStatus::Unavailable,
            max_dual_infeasibility: None,
            primal_dual_objective_error: None,
        }
    }
    #[test]
    fn quality_reads_typed_evidence_only() {
        let accuracy = ResolvedAccuracy {
            mip_relative_gap: 0.03,
            ..ResolvedAccuracy::nominal()
        };
        // Observational metrics under the former keys grant nothing.
        let mut r = report(Backend::Highs, Termination::Success);
        r.metrics.extend([
            ("upload.equivalent".into(), Metric::Bool(true)),
            ("model.discrete".into(), Metric::Bool(true)),
            ("mip_gap".into(), Metric::Real(0.02)),
            ("mip_dual_bound".into(), Metric::Real(9.8)),
            ("objective_function_value".into(), Metric::Real(10.0)),
        ]);
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Feasible);
        // Typed evidence qualifies without any metric.
        r.metrics.clear();
        r.evidence.coefficient = Some(CoefficientEvidence {
            upload_equivalent: false,
            ..coefficient()
        });
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Feasible);
        r.evidence.coefficient = Some(coefficient());
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::GapQualified);
        // Continuous LP optimality needs a typed feasible dual status.
        let continuous = CoefficientEvidence {
            discrete: false,
            max_dual_infeasibility: Some(0.0),
            primal_dual_objective_error: Some(0.0),
            ..coefficient()
        };
        r.evidence.coefficient = Some(continuous);
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Feasible);
        r.evidence.coefficient = Some(CoefficientEvidence {
            dual: SolutionStatus::Feasible,
            ..continuous
        });
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::OptimalWithinTolerance);
        // KKT metrics alone never make an NLP stationary.
        let mut nlp = report(Backend::Ipopt, Termination::Success);
        let mut o = Observation::from_values(Some(10.0), vec![], vec![]).unwrap();
        o.dual_error = None;
        nlp.observation = Some(o);
        nlp.metrics.extend([
            ("quality.stationarity.accepted".into(), Metric::Bool(true)),
            (
                "quality.complementarity.accepted".into(),
                Metric::Bool(true),
            ),
        ]);
        qualify(&mut nlp, &accuracy);
        assert_eq!(nlp.qualification, Qualification::Feasible);
        nlp.evidence.kkt = Some(KktEvidence {
            stationarity: Some(true),
            complementarity: Some(true),
        });
        qualify(&mut nlp, &accuracy);
        assert_eq!(nlp.qualification, Qualification::Stationary);
        // Conic residuals come from typed evidence.
        let mut conic = report(Backend::Clarabel, Termination::Success);
        conic.metrics.extend([
            ("res_primal".into(), Metric::Real(0.0)),
            ("res_dual".into(), Metric::Real(0.0)),
            ("gap_abs".into(), Metric::Real(0.0)),
        ]);
        qualify(&mut conic, &accuracy);
        assert_eq!(conic.qualification, Qualification::Feasible);
        conic.evidence.conic = Some(ConicEvidence {
            primal_residual: 0.0,
            dual_residual: 0.0,
            gap_absolute: 0.0,
            gap_relative: 0.0,
        });
        qualify(&mut conic, &accuracy);
        assert_eq!(conic.qualification, Qualification::OptimalWithinTolerance);
    }
    #[test]
    fn gaps_and_limits_have_separate_meanings() {
        let mut r = report(Backend::Highs, Termination::Success);
        r.evidence.coefficient = Some(coefficient());
        let accuracy = ResolvedAccuracy {
            mip_relative_gap: 0.03,
            ..ResolvedAccuracy::nominal()
        };
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
        r.evidence.coefficient = Some(CoefficientEvidence {
            mip_gap: Some(f64::NAN),
            ..coefficient()
        });
        qualify(&mut r, &accuracy);
        assert_eq!(r.qualification, Qualification::Feasible);
    }
}
