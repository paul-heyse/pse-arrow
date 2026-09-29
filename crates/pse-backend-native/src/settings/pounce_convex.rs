// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The POUNCE-convex settings document (Plan 22 N5): the interior-point method's typed
//! choices and the complete native FERAL configuration. Its stopping tolerance comes from
//! the resolved accuracy and its iteration budget from the shared controls; nothing here
//! restates them.

/// The POUNCE-convex adapter's settings type. Its identity derives from serde.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "PounceConvexSettings")]
pub struct Settings {
    /// Solve through the homogeneous self-dual embedding, which detects primal and dual
    /// infeasibility from the iterates, instead of the direct infeasible-start method.
    pub self_dual: bool,
    /// Ruiz-equilibrate the data before the solve.
    pub equilibrate: bool,
    /// Cross over from the interior-point solution to a vertex of a linear program.
    pub crossover: bool,
    /// Native linear settings. A batch factors each instance serially: its parallelism is
    /// across instances.
    #[serde(with = "super::pounce::FeralIdentity")]
    pub linear: super::pounce::LinearSettings,
}
impl Default for Settings {
    /// The library's own defaults: the self-dual embedding, equilibration, no crossover and
    /// FERAL's defaults.
    fn default() -> Self {
        Self {
            self_dual: true,
            equilibrate: true,
            crossover: false,
            linear: super::pounce::LinearSettings::default(),
        }
    }
}
