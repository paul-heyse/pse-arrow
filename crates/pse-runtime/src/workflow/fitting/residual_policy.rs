// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed residual-LM algorithm criteria, independent of scientific publication budgets.

/// Exact algorithm stopping criteria used by residual LM.
/// These relative/cosine values do not represent original physical stationarity.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ResidualLmCriteria {
    /// Encoding of these distinct algorithm criteria.
    version: pse_model::document::Version<1>,
    /// Actual and predicted relative objective-reduction threshold (ftol).
    relative_cost_change: f64,
    /// Relative consecutive-iterate threshold (xtol).
    relative_step: f64,
    /// Maximum normalized residual/Jacobian-column cosine (gtol).
    column_residual_cosine: f64,
}
impl ResidualLmCriteria {
    /// The fixed current algorithm policy, matching the pinned non-MINPACK defaults.
    pub const fn current() -> Self {
        Self {
            version: pse_model::document::Version,
            relative_cost_change: 30. * f64::EPSILON,
            relative_step: 30. * f64::EPSILON,
            column_residual_cosine: 30. * f64::EPSILON,
        }
    }
    /// Required version of the recorded algorithm-criteria contract.
    pub const fn version(self) -> u32 {
        pse_model::document::Version::<1>::NUMBER
    }
    /// Actual relative objective-reduction threshold.
    pub const fn relative_cost_change(self) -> f64 {
        self.relative_cost_change
    }
    /// Actual relative consecutive-iterate threshold.
    pub const fn relative_step(self) -> f64 {
        self.relative_step
    }
    /// Actual dimensionless residual/Jacobian-column cosine threshold.
    pub const fn column_residual_cosine(self) -> f64 {
        self.column_residual_cosine
    }
    pub(super) fn algorithm(self) -> levenberg_marquardt::LevenbergMarquardt<f64> {
        levenberg_marquardt::LevenbergMarquardt::new()
            .with_ftol(self.relative_cost_change)
            .with_xtol(self.relative_step)
            .with_gtol(self.column_residual_cosine)
    }
    // A retained comparison inside the actual driver, never a production request.
    #[cfg(all(test, feature = "solver-ipopt"))]
    pub(super) fn inquiry_baseline(stationarity: f64) -> Self {
        Self {
            relative_cost_change: stationarity,
            relative_step: stationarity,
            column_residual_cosine: 0.,
            ..Self::current()
        }
    }
}

/// Actual algorithm criteria and finite allowances for one executed LM invocation.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ResidualLmStopping {
    /// The actual distinct algorithm stopping criteria.
    pub criteria: ResidualLmCriteria,
    /// Conservative maximum accepted parameter-update calls.
    pub parameter_update_limit: usize,
    /// Maximum residual callback calls, checked before evaluating the original model.
    pub residual_evaluation_limit: usize,
}
