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
use native::{
    ProblemError,
    kkt::Withheld,
    solve::{Qualification, SolveReport},
};
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
                write!(f, "observations with an importance other than one: {}", ids(o))
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
    /// `Σ` row-major, in row-parameter unit × column-parameter unit, or why it is withheld.
    pub values: Result<Vec<f64>, FitWithheld>,
}
impl Covariance {
    /// Retained cells.
    pub(super) fn cells(&self) -> usize {
        self.values.as_ref().map_or(0, Vec::len)
    }
    /// The standard error of each parameter, `√Σₖₖ`, when certified.
    pub fn standard_errors(&self) -> Option<Vec<f64>> {
        let values = self.values.as_ref().ok()?;
        let n = self.parameters.len();
        Some((0..n).map(|k| values[k * n + k].max(0.0).sqrt()).collect())
    }
}

/// One end of a confidence interval.
#[derive(Clone, Debug, PartialEq)]
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

/// Whether a native report's candidate is qualified stationary or better.
fn stationary(solve: &SolveReport) -> Result<(), Withheld> {
    match solve.qualification {
        Qualification::Stationary
        | Qualification::OptimalWithinTolerance
        | Qualification::GapQualified => Ok(()),
        other => Err(Withheld::Unqualified(other)),
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
        let solve = match (&report.candidate, &report.solve) {
            (Some(_), Some(solve)) => solve,
            _ => return Err(FitWithheld::Local(Withheld::NoCandidate)),
        };
        // Admission refuses an included observation without a declared deviation, so the
        // model's remaining condition is unit importance.
        let weighted: Vec<_> = self
            .measurements
            .iter()
            .filter(|o| o.included && o.importance != 1.0)
            .map(|o| o.id)
            .collect();
        if !weighted.is_empty() {
            return Err(FitWithheld::NonunitImportance(weighted));
        }
        stationary(solve).map_err(FitWithheld::Local)?;
        if report.objective.is_none() {
            return Err(FitWithheld::Responses(report.diagnostic.clone()));
        }
        Ok(())
    }
    /// The fit's one covariance, certified or withheld; `None` without free parameters.
    pub(super) fn covariance(&self, report: &FitReport) -> Option<Covariance> {
        let parameters = self.free_ids();
        if parameters.is_empty() {
            return None;
        }
        let approximation = if self.profile.solver.controls.hessian == HessianMode::Exact {
            CovarianceApproximation::Exact
        } else {
            CovarianceApproximation::GaussNewton
        };
        let values = self.covariance_values(report, approximation);
        Some(Covariance {
            approximation,
            parameters,
            values,
        })
    }
    fn covariance_values(
        &self,
        report: &FitReport,
        approximation: CovarianceApproximation,
    ) -> Result<Vec<f64>, FitWithheld> {
        self.estimate_valid(report)?;
        let np = self.free().count();
        let (Some(rank), Some(directions)) = (report.rank, report.directions.as_ref()) else {
            return Err(FitWithheld::Responses(report.diagnostic.clone()));
        };
        if rank < np {
            return Err(FitWithheld::RankDeficient {
                rank,
                parameters: np,
            });
        }
        let candidate = report
            .candidate
            .as_ref()
            .ok_or(FitWithheld::Local(Withheld::NoCandidate))?;
        let held: Vec<_> = self
            .free()
            .filter(|(k, col)| {
                let parameter = &self.declaration.parameters[*k];
                let tolerance = self.tolerances.variables[col.get()];
                let value = candidate[col.get()];
                parameter.lower.is_some_and(|b| value - b <= tolerance)
                    || parameter.upper.is_some_and(|b| b - value <= tolerance)
            })
            .map(|(k, _)| self.declaration.parameters[k].symbol_id)
            .collect();
        if !held.is_empty() {
            return Err(FitWithheld::AtBound(held));
        }
        match approximation {
            CovarianceApproximation::Exact => {
                let solve = report
                    .solve
                    .as_ref()
                    .ok_or(FitWithheld::Local(Withheld::NoCandidate))?;
                match &solve.evidence.inverse_reduced_hessian {
                    Some(Ok(block)) if block.values.iter().any(|v| !v.is_finite()) => {
                        Err(FitWithheld::Local(Withheld::Backsolve))
                    }
                    Some(Ok(block)) if block.columns.len() == np => Ok(block.values.clone()),
                    Some(Ok(_)) | None => Err(FitWithheld::Local(Withheld::Analysis(
                        native::kkt::Unavailable::from(ProblemError::internal(
                            "the fit's analysis read no inverse reduced Hessian over its parameters",
                        )),
                    ))),
                    Some(Err(withheld)) => Err(FitWithheld::Local(withheld.clone())),
                }
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
                if values.iter().any(|v| !v.is_finite()) {
                    return Err(FitWithheld::Local(Withheld::Backsolve));
                }
                Ok(values)
            }
        }
    }
    /// Wald intervals `θ̂ₖ ± z·√Σₖₖ` with `z` the standard normal quantile of
    /// `(1 + level)/2`; withheld when the covariance is.
    pub(super) fn wald(
        &self,
        report: &FitReport,
        level: f64,
    ) -> Result<Vec<Interval>, FitWithheld> {
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
    }
    /// The quantities the fit derives from its estimate: the covariance, and on request
    /// Wald intervals and profile-likelihood chains.
    pub(super) fn derive(
        self: &Arc<Self>,
        report: &mut FitReport,
        route: native::routing::Route,
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
            report.profiles = Some(self.profiles(
                report,
                level,
                controls,
                (route, workers),
                execution,
            ));
        }
    }
}

#[cfg(test)]
#[path = "covariance_tests.rs"]
mod tests;
