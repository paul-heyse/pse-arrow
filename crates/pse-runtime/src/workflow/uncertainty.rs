// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Propagation of a parameter covariance to outputs, `Σ_y = J·Σ_θ·Jᵀ` (Plan 22 S4; ADR-0118
//! items 1, 10 and 11; PS-12): the counterpart of IDAES `sens.py`.
//!
//! `J` is either a modeling step's parametric sensitivities (S1), whose parameters are
//! matched to the covariance's by identity, or a fit's own response derivatives. The
//! result is a first-order statement that holds while both inputs do, so its validity is
//! the conjunction of theirs: when either is withheld, so is the propagated covariance,
//! with `upstream_withheld` and the upstream reason in its detail.
//!
//! A caller propagates a fit's covariance through a later solve by naming it in the solve
//! settings (`SensitivityRequest.propagation`, built from the fit's
//! `runtime.parameter_covariances` row or [`RunResult::parameter_covariance`]); a fit
//! propagates to its own predictions on request (`FitUncertainty.predictions`). Both
//! publish `runtime.propagated_covariances` with a `local_validity` row.
use super::{RunReport, RunResult, WorkflowError, contract};
pub use crate::math::settings::ParameterCovariance;
use pse_ids::SemanticId;
use pse_model::scalars::FiniteBound;
use pse_relations::generated::{
    enums::{DerivedQuantity, WithheldReason},
    identities::RunId,
};

/// Output derivatives with respect to parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Jacobian {
    /// The outputs, by identity: the rows.
    pub outputs: Vec<SemanticId>,
    /// The parameters, by identity: the columns.
    pub parameters: Vec<SemanticId>,
    /// `J` row-major, in output unit per parameter unit.
    pub values: Vec<f64>,
}
/// An input of the propagation that was withheld, and why (PS-12).
#[derive(Clone, Debug, PartialEq)]
pub struct Upstream {
    /// The withheld quantity.
    pub quantity: DerivedQuantity,
    /// Its registry reason.
    pub reason: WithheldReason,
    /// Its typed cause.
    pub detail: String,
}
impl std::fmt::Display for Upstream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the {} it is computed from was withheld ({}): {}",
            self.quantity.as_str(),
            self.reason.as_str(),
            self.detail
        )
    }
}
/// A certified propagated covariance.
#[derive(Clone, Debug, PartialEq)]
pub struct Propagated {
    /// The fit run whose covariance was propagated.
    pub covariance_run: RunId,
    /// The parameters propagated, in the covariance's order.
    pub parameters: Vec<SemanticId>,
    /// The outputs: the order of every matrix.
    pub outputs: Vec<SemanticId>,
    /// `Σ_y` row-major, in row-output unit × column-output unit.
    pub values: Vec<f64>,
}

/// `Σ_y = J·Σ_θ·Jᵀ` over the covariance's parameters, each matched by identity to a column
/// of `J`; a parameter of `J` the covariance does not name is held fixed. Withheld, with
/// the first withheld input, when an input is withheld.
///
/// # Errors
/// A covariance that is not square over distinct parameters, a `J` whose dimensions do not
/// match its identities, or a covariance parameter `J` does not differentiate.
pub fn propagate(
    covariance: Result<&ParameterCovariance, Upstream>,
    jacobian: Result<&Jacobian, Upstream>,
) -> Result<Result<Propagated, Upstream>, WorkflowError> {
    let (covariance, jacobian) = match (covariance, jacobian) {
        (Ok(c), Ok(j)) => (c, j),
        (Err(upstream), _) | (_, Err(upstream)) => return Ok(Err(upstream)),
    };
    covariance
        .admit()
        .map_err(|e| contract(format!("propagated covariance: {e}")))?;
    let (m, n) = (jacobian.outputs.len(), jacobian.parameters.len());
    if jacobian.values.len() != m * n {
        return Err(contract(
            "a Jacobian's values match its outputs and parameters",
        ));
    }
    let p = covariance.parameters.len();
    let columns = covariance
        .parameters
        .iter()
        .map(|id| {
            jacobian
                .parameters
                .iter()
                .position(|q| q == id)
                .ok_or_else(|| contract(format!("the Jacobian does not differentiate {id}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let sigma = |k: usize, l: usize| covariance.values[k * p + l].into_inner();
    let j = |i: usize, k: usize| jacobian.values[i * n + columns[k]];
    // J·Σ_θ, then (J·Σ_θ)·Jᵀ.
    let product: Vec<f64> = (0..m * p)
        .map(|index| {
            let (i, l) = (index / p, index % p);
            (0..p).map(|k| j(i, k) * sigma(k, l)).sum()
        })
        .collect();
    let values: Vec<f64> = (0..m * m)
        .map(|index| {
            let (i, r) = (index / m, index % m);
            (0..p).map(|l| product[i * p + l] * j(r, l)).sum()
        })
        .collect();
    if values.iter().any(|v| !v.is_finite()) {
        return Err(contract("the propagated covariance overflowed"));
    }
    Ok(Ok(Propagated {
        covariance_run: covariance.run_id,
        parameters: covariance.parameters.clone(),
        outputs: jacobian.outputs.clone(),
        values,
    }))
}

impl RunResult {
    /// This fit run's parameter covariance as a propagation input naming the run, or why it
    /// is withheld; `None` for a run that is not a fit with free parameters.
    pub fn parameter_covariance(&self) -> Option<Result<ParameterCovariance, Upstream>> {
        let RunReport::Fit(report) = self.report.as_ref().ok()? else {
            return None;
        };
        report
            .covariance
            .as_ref()
            .map(|c| c.propagation_input(self.run_id))
    }
}
impl super::Covariance {
    /// The covariance as a propagation input naming `run`, or why it is withheld.
    pub(crate) fn propagation_input(&self, run: RunId) -> Result<ParameterCovariance, Upstream> {
        match &self.values {
            Ok(values) => Ok(ParameterCovariance {
                run_id: run,
                parameters: self.parameters.clone(),
                values: values
                    .iter()
                    .map(|v| FiniteBound::try_new(*v))
                    .collect::<Result<_, _>>()
                    .map_err(|e| Upstream {
                        quantity: DerivedQuantity::ParameterCovariance,
                        reason: WithheldReason::BacksolveFailed,
                        detail: e.to_string(),
                    })?,
            }),
            Err(withheld) => Err(Upstream {
                quantity: DerivedQuantity::ParameterCovariance,
                reason: withheld.reason(),
                detail: withheld.to_string(),
            }),
        }
    }
}

#[cfg(all(test, feature = "solver-ipopt", feature = "solver-highs"))]
#[path = "uncertainty_tests.rs"]
mod tests;
