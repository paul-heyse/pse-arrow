// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

const CONFIDENCE_DOMAIN: pse_model::boundary_projection::Float64Domain =
    pse_model::boundary_projection::Float64Domain::Interval(
        pse_model::boundary_projection::Float64Interval::new(0.0, false, 1.0, false));

pse_model::projection_struct! {
/// Confidence intervals and optional profile/prediction work of the actual fit.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FitUncertainty {
    projection: { name: "FitUncertainty", rust_path: crate::workflow::FitUncertainty, version: 1,
        direction: pse_model::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// The confidence level of the intervals, below one.
    pub level: Fraction => { tag: 1, direction: pse_model::boundary_projection::Direction::Both,
        rules: [pse_model::boundary_projection::LocalRule::Float64Domain(CONFIDENCE_DOMAIN)] };
    /// Also derive profile-likelihood intervals under these controls; absent derives Wald
    /// intervals alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<ProfileControls> => { tag: 2, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Also propagate the covariance to the included observations' predictions,
    /// `Σ_y = R·Σ_θ·Rᵀ` over the fit's response derivatives (Plan 22 S4); their count
    /// squared is bounded by the profile's `max_cells`.
    #[serde(default)]
    pub predictions: bool => { tag: 3, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
}
}

pse_model::projection_struct! {
/// Existing bounded profile-likelihood pin-chain controls.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProfileControls {
    projection: { name: "FitProfileControls", rust_path: crate::workflow::ProfileControls, version: 1,
        direction: pse_model::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Pinned fits at most per chain, refinements and failed fits included.
    #[serde(default = "ProfileControls::default_points")]
    pub points: PositiveCount => { tag: 1, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Relative tolerance on the signed-root statistic at a threshold end.
    #[serde(default = "ProfileControls::default_tolerance")]
    pub tolerance: Fraction => { tag: 2, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Chains solved at once, each pinned fit on the fit's own threads. Their phase admits
    /// the team after estimation relinquishes active capacity. Absent uses available cores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workers: Option<PositiveCount> => { tag: 3, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
}
}
