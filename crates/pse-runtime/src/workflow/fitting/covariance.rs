// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The covariance of a fit's estimate and its Wald intervals (Plan 22 S3; ADR-0118 items
//! 2, 3 and 8; PS-12).
//!
//! **One covariance per fit.** When the fit used the exact Hessian, the covariance is the
//! inverse reduced Hessian over its parameter columns, `B·K⁻¹·Bᵀ` of the fit's own KKT
//! analysis (`kkt::InverseReducedHessian`). Otherwise it is Gauss–Newton,
//! `Σ = S·V·diag(s⁻²)·Vᵀ·S` from the singular value decomposition `W = U·diag(s)·Vᵀ` of the
//! weighted, parameter-scaled response matrix `W = R·S/σ` the rank diagnostic already
//! computes, with `S` the declared parameter scales. `JᵀWJ` is never formed: it would
//! square the condition number. The objective is half the weighted residual sum of
//! squares, the negative log-likelihood of the declared model, so its Hessian is the
//! information and needs no factor.
//!
//! **Statistical model.** The declared standard deviations are absolute, so no residual
//! variance estimate rescales the covariance, and the Wald quantile is the standard
//! normal's. Admission already refuses an included observation without a declared
//! deviation; a fit whose included observations carry an importance other than one has no
//! covariance (F02).
//!
//! **Validity.** The covariance is withheld, with the first failed condition recorded,
//! unless the fit has a candidate qualified stationary or better, the declared model holds
//! for every included observation, the responses have full rank, and no free parameter
//! lies at a declared bound; the exact covariance also needs the fit's KKT point to have
//! independent active gradients, strict complementarity and second-order sufficiency.
use super::*;
use native::{ProblemError, kkt::Withheld, solve::Qualification};
use pse_relations::generated::enums::{
    CovarianceApproximation, DerivedQuantity, IntervalOutcome, WithheldReason,
};
use statrs::distribution::{ContinuousCDF, Normal};

/// Why a fit's derived quantity is withheld (PS-12).
#[derive(Clone, Debug)]
pub enum FitWithheld {
    /// A condition of the candidate or of the fit's KKT-point analysis.
    Local(Withheld),
    /// Included observations with an importance other than one.
    NonunitImportance(Vec<SemanticId>),
    /// The local responses are unavailable, so the response rank is unknown.
    Responses(Option<FitDiagnostic>),
    /// The responses do not have full rank; `FitReport::directions` holds the identifiable
    /// directions.
    RankDeficient {
        /// The numerical rank.
        rank: usize,
        /// The free parameters.
        parameters: usize,
    },
    /// Free parameters at a declared bound.
    AtBound(Vec<SemanticId>),
    /// A quantity this one is computed from was withheld.
    Upstream(DerivedQuantity),
}
impl FitWithheld {
    /// The registry reason.
    pub fn reason(&self) -> WithheldReason {
        match self {
            Self::Local(withheld) => crate::workflow::local_analysis::reason(withheld),
            Self::NonunitImportance(_) => WithheldReason::NonunitImportance,
            Self::Responses(_) => WithheldReason::ResponsesUnavailable,
            Self::RankDeficient { .. } => WithheldReason::RankDeficient,
            Self::AtBound(_) => WithheldReason::ParameterAtBound,
            Self::Upstream(_) => WithheldReason::UpstreamWithheld,
        }
    }
}
impl std::fmt::Display for FitWithheld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids = |ids: &[SemanticId]| {
            ids.iter()
                .map(|id| id.to_hex())
                .collect::<Vec<_>>()
                .join(", ")
        };
        match self {
            Self::Local(withheld) => write!(f, "{withheld}"),
            Self::NonunitImportance(o) => {
                write!(
                    f,
                    "observations with an importance other than one: {}",
                    ids(o)
                )
            }
            Self::Responses(Some(d)) => write!(f, "local responses unavailable: {d}"),
            Self::Responses(None) => f.write_str("local responses unavailable"),
            Self::RankDeficient { rank, parameters } => write!(
                f,
                "the responses have rank {rank} over {parameters} free parameters"
            ),
            Self::AtBound(p) => write!(f, "parameters at a declared bound: {}", ids(p)),
            Self::Upstream(q) => write!(f, "the {} it is computed from was withheld", q.as_str()),
        }
    }
}

/// The covariance of a fit's free parameters (ADR-0118 item 8).
#[derive(Clone, Debug)]
pub struct Covariance {
    /// Exact from the fit's KKT analysis when it used the exact Hessian, Gauss–Newton
    /// otherwise.
    pub approximation: CovarianceApproximation,
    /// The free parameters in fit order: the order of every matrix.
    pub parameters: Vec<SemanticId>,
    /// Recorded free-parameter scales in the matrix's physical units.
    pub scales: Vec<f64>,
    /// Complete parameter quantity contracts in free-parameter order.
    pub parameter_quantities: Vec<SemanticId>,
    /// Actual representation units of the covariance and scales.
    pub parameter_units: Vec<SemanticId>,
    /// `Σ` row-major, in row-parameter unit × column-parameter unit, or why it is withheld.
    pub values: Result<Vec<f64>, FitWithheld>,
}
impl Covariance {
    /// Retained cells.
    pub(super) fn cells(&self) -> usize {
        self.values
            .as_ref()
            .map_or(0, Vec::capacity)
            .saturating_add(self.scales.capacity())
            .saturating_add(self.parameters.capacity().saturating_mul(2))
            .saturating_add(self.parameter_quantities.capacity().saturating_mul(2))
            .saturating_add(self.parameter_units.capacity().saturating_mul(2))
    }
    /// The standard error of each parameter, `√Σₖₖ`, when certified.
    pub fn standard_errors(&self) -> Option<Vec<f64>> {
        let values = self.values.as_ref().ok()?;
        let n = self.parameters.len();
        if n.checked_mul(n) != Some(values.len())
            || (0..n).any(|index| {
                !values[index * n + index].is_finite() || values[index * n + index] < 0.0
            })
        {
            return None;
        }
        Some((0..n).map(|k| values[k * n + k].sqrt()).collect())
    }
}

/// One end of a confidence interval.
#[derive(Clone, Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IntervalBound {
    /// The end in the parameter's unit; absent when a profile chain stopped.
    pub value: Option<f64>,
    /// How the end was reached.
    pub outcome: IntervalOutcome,
}
/// A confidence interval of one free parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct Interval {
    /// The parameter.
    pub parameter: SemanticId,
    /// Its estimate.
    pub estimate: f64,
    /// The lower end.
    pub lower: IntervalBound,
    /// The upper end.
    pub upper: IntervalBound,
}

/// Statistical and qualification facts; no runtime or prepared model is needed to
/// decide whether an estimate can support a derived quantity.
struct EstimateEvidence<'a> {
    candidate: bool,
    residual_qualification: Option<Qualification>,
    native_qualification: Option<Qualification>,
    objective: bool,
    diagnostic: &'a Option<FitDiagnostic>,
}
fn estimate_policy(
    evidence: EstimateEvidence<'_>,
    nonunit_importance: impl Iterator<Item = SemanticId>,
) -> Result<(), FitWithheld> {
    if !evidence.candidate {
        return Err(FitWithheld::Local(Withheld::NoCandidate));
    }
    if let Some(qualification) = evidence.residual_qualification {
        if qualification != Qualification::Stationary {
            return Err(FitWithheld::Local(Withheld::Unqualified(qualification)));
        }
    } else {
        match evidence.native_qualification {
            Some(
                Qualification::Stationary
                | Qualification::OptimalWithinTolerance
                | Qualification::GapQualified,
            ) => {}
            Some(other) => return Err(FitWithheld::Local(Withheld::Unqualified(other))),
            None => return Err(FitWithheld::Local(Withheld::NoCandidate)),
        }
    }
    // Admission already requires a deviation for every included observation.
    let weighted: Vec<_> = nonunit_importance.collect();
    if !weighted.is_empty() {
        return Err(FitWithheld::NonunitImportance(weighted));
    }
    if !evidence.objective {
        return Err(FitWithheld::Responses(evidence.diagnostic.clone()));
    }
    Ok(())
}
fn response_policy(
    rank: Option<usize>,
    directions: bool,
    parameters: usize,
    diagnostic: &Option<FitDiagnostic>,
) -> Result<(), FitWithheld> {
    let Some(rank) = rank.filter(|_| directions) else {
        return Err(FitWithheld::Responses(diagnostic.clone()));
    };
    if rank < parameters {
        return Err(FitWithheld::RankDeficient { rank, parameters });
    }
    Ok(())
}
struct ParameterBoundEvidence {
    id: SemanticId,
    column: usize,
    bounds: (f64, f64),
    tolerance: f64,
    scale: f64,
}
enum BoundEvidence<'a> {
    Residual {
        finite_variable_bounds: bool,
        general_bounds: bool,
    },
    Native {
        candidate: &'a [f64],
        duals: Option<(&'a [f64], &'a [f64])>,
        objective_scale: f64,
        stationarity: f64,
    },
}
fn bound_policy(
    evidence: BoundEvidence<'_>,
    parameters: impl Iterator<Item = ParameterBoundEvidence>,
) -> Result<(), FitWithheld> {
    let held = match evidence {
        BoundEvidence::Residual {
            finite_variable_bounds,
            general_bounds,
        } => {
            if finite_variable_bounds || general_bounds {
                return Err(FitWithheld::Local(Withheld::Multipliers));
            }
            Vec::new()
        }
        BoundEvidence::Native {
            candidate,
            duals,
            objective_scale,
            stationarity,
        } => {
            let (lower, upper) = duals.ok_or(FitWithheld::Local(Withheld::Multipliers))?;
            if lower.len() != candidate.len() || upper.len() != candidate.len() {
                return Err(FitWithheld::Local(Withheld::Multipliers));
            }
            parameters
                .filter(|p| {
                    native::kkt::bound_activity(
                        candidate[p.column],
                        p.bounds,
                        p.tolerance,
                        (lower[p.column], upper[p.column]),
                        (p.scale, objective_scale),
                        stationarity,
                    )
                    .is_active()
                })
                .map(|p| p.id)
                .collect()
        }
    };
    if held.is_empty() {
        Ok(())
    } else {
        Err(FitWithheld::AtBound(held))
    }
}
fn exact_covariance_policy(
    inverse: Option<&Result<native::kkt::InverseReducedHessian, Withheld>>,
    parameters: usize,
) -> Result<&[f64], FitWithheld> {
    match inverse {
        Some(Ok(block)) if block.values.iter().any(|v| !v.is_finite()) => {
            Err(FitWithheld::Local(Withheld::Backsolve))
        }
        Some(Ok(block)) if block.columns.len() == parameters => Ok(&block.values),
        Some(Ok(_)) | None => Err(FitWithheld::Local(Withheld::Analysis(
            native::kkt::Unavailable::from(ProblemError::internal(
                "the fit's analysis read no inverse reduced Hessian over its parameters",
            )),
        ))),
        Some(Err(withheld)) => Err(FitWithheld::Local(withheld.clone())),
    }
}
fn finite_covariance_policy(values: &[f64]) -> Result<(), FitWithheld> {
    if values.iter().any(|v| !v.is_finite()) {
        Err(FitWithheld::Local(Withheld::Backsolve))
    } else {
        Ok(())
    }
}

impl FitProblem {
    /// The free parameters' identities, in fit order.
    fn free_ids(&self) -> Vec<SemanticId> {
        self.free()
            .map(|(k, _)| self.declaration.parameters[k].symbol_id)
            .collect()
    }
    /// The conditions every derived quantity of the estimate shares: a candidate qualified
    /// stationary or better, and the declared statistical model (F02).
    pub(super) fn estimate_valid(&self, report: &FitReport) -> Result<(), FitWithheld> {
        estimate_policy(
            EstimateEvidence {
                candidate: report.candidate.is_some(),
                residual_qualification: report.residual.as_ref().map(|r| r.qualification),
                native_qualification: report.solve.as_ref().map(|s| s.qualification),
                objective: report.objective.is_some(),
                diagnostic: &report.diagnostic,
            },
            self.measurements
                .iter()
                .filter(|o| o.included && o.importance != 1.0)
                .map(|o| o.id),
        )
    }
    /// The fit's one covariance, certified or withheld; `None` without free parameters.
    pub(super) fn covariance(&self, report: &FitReport) -> Option<Covariance> {
        let parameters = self.free_ids();
        if parameters.is_empty() {
            return None;
        }
        let approximation = if report.residual.is_none()
            && self.profile.solver.controls.hessian == HessianMode::Exact
        {
            CovarianceApproximation::Exact
        } else {
            CovarianceApproximation::GaussNewton
        };
        let values = self.covariance_values(report, approximation);
        Some(Covariance {
            approximation,
            parameters,
            scales: self
                .free()
                .map(|(k, _)| self.declaration.parameters[k].scale)
                .collect(),
            parameter_quantities: self
                .free()
                .map(|(k, _)| self.parameter_ports[k].quantity.as_id())
                .collect(),
            parameter_units: self
                .free()
                .map(|(k, _)| self.parameter_ports[k].unit.as_id())
                .collect(),
            values,
        })
    }
    fn covariance_values(
        &self,
        report: &FitReport,
        approximation: CovarianceApproximation,
    ) -> Result<Vec<f64>, FitWithheld> {
        crate::workflow::dynamics::analysis("fit.covariance_analysis", || {
            self.estimate_valid(report)?;
            let np = self.free().count();
            response_policy(
                report.rank,
                report.directions.is_some(),
                np,
                &report.diagnostic,
            )?;
            let directions = report
                .directions
                .as_ref()
                .ok_or_else(|| FitWithheld::Responses(report.diagnostic.clone()))?;
            let candidate = report
                .candidate
                .as_ref()
                .ok_or(FitWithheld::Local(Withheld::NoCandidate))?;
            let bounds = if report.residual.is_some() {
                BoundEvidence::Residual {
                    finite_variable_bounds: self
                        .contract
                        .variables
                        .iter()
                        .any(|v| v.lower.is_finite() || v.upper.is_finite()),
                    general_bounds: !self.bounds.is_empty(),
                }
            } else {
                BoundEvidence::Native {
                    candidate,
                    duals: report
                        .solve
                        .as_ref()
                        .and_then(|s| s.candidate.as_ref())
                        .and_then(|c| c.bound_dual.as_ref())
                        .map(|(l, u)| (l.as_slice(), u.as_slice())),
                    objective_scale: self.normalization.objective,
                    stationarity: self.accuracy.stationarity,
                }
            };
            bound_policy(
                bounds,
                self.free().map(|(k, col)| {
                    let parameter = &self.declaration.parameters[k];
                    let column = col.get();
                    ParameterBoundEvidence {
                        id: parameter.symbol_id,
                        column,
                        bounds: (
                            parameter.lower.unwrap_or(f64::NEG_INFINITY),
                            parameter.upper.unwrap_or(f64::INFINITY),
                        ),
                        tolerance: self.tolerances.variables[column],
                        scale: self.normalization.variables[column],
                    }
                }),
            )?;
            match approximation {
                CovarianceApproximation::Exact => {
                    let solve = report
                        .solve
                        .as_ref()
                        .ok_or(FitWithheld::Local(Withheld::NoCandidate))?;
                    exact_covariance_policy(solve.evidence.inverse_reduced_hessian.as_ref(), np)
                        .map(<[f64]>::to_vec)
                }
                CovarianceApproximation::GaussNewton => {
                    // Σ = S·V·diag(s⁻²)·Vᵀ·S over the full-rank spectrum.
                    let s = &report.singular_values;
                    let scales: Vec<f64> = self
                        .free()
                        .map(|(k, _)| self.declaration.parameters[k].scale)
                        .collect();
                    let mut values = vec![0.0; np * np];
                    for i in 0..np {
                        for j in 0..np {
                            let scaled: f64 = (0..np)
                                .map(|k| directions[(i, k)] * directions[(j, k)] / (s[k] * s[k]))
                                .sum();
                            values[i * np + j] = scales[i] * scaled * scales[j];
                        }
                    }
                    finite_covariance_policy(&values)?;
                    Ok(values)
                }
            }
        })
    }
    /// Wald intervals `θ̂ₖ ± z·√Σₖₖ` with `z` the standard normal quantile of
    /// `(1 + level)/2`; withheld when the covariance is.
    pub(super) fn wald(
        &self,
        report: &FitReport,
        level: f64,
    ) -> Result<Vec<Interval>, FitWithheld> {
        crate::workflow::dynamics::analysis("fit.wald_interval_analysis", || {
            let covariance = report
                .covariance
                .as_ref()
                .ok_or(FitWithheld::Upstream(DerivedQuantity::ParameterCovariance))?;
            let errors = covariance
                .standard_errors()
                .ok_or(FitWithheld::Upstream(DerivedQuantity::ParameterCovariance))?;
            let candidate = report
                .candidate
                .as_ref()
                .ok_or(FitWithheld::Local(Withheld::NoCandidate))?;
            let z = Normal::standard().inverse_cdf(0.5 * (1.0 + level));
            Ok(self
                .free()
                .zip(errors)
                .map(|((k, col), error)| {
                    let estimate = candidate[col.get()];
                    let end = |value: f64| IntervalBound {
                        value: Some(value),
                        outcome: IntervalOutcome::Threshold,
                    };
                    Interval {
                        parameter: self.declaration.parameters[k].symbol_id,
                        estimate,
                        lower: end(estimate - z * error),
                        upper: end(estimate + z * error),
                    }
                })
                .collect())
        })
    }
    /// The quantities the fit derives from its estimate: the covariance, and on request
    /// Wald intervals and profile-likelihood chains.
    pub(super) fn derive(
        self: &Arc<Self>,
        report: &mut FitReport,
        route: FitRoute,
        execution: &native::solve::Execution,
        workers: usize,
    ) {
        report.covariance = self.covariance(report);
        let Some(uncertainty) = &self.profile.uncertainty else {
            return;
        };
        if report.covariance.is_none() {
            return;
        }
        let level = uncertainty.level.into_inner();
        report.wald = Some(self.wald(report, level));
        if let Some(controls) = &uncertainty.profile {
            report.profiles =
                Some(self.profiles(report, level, controls, (route, workers), execution));
        }
    }
}

#[cfg(test)]
#[path = "covariance_tests.rs"]
mod tests;
