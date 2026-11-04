// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pse_contract::projection_struct! {
    /// The POUNCE-convex adapter's settings type. Its identity derives from serde.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(default, deny_unknown_fields)]
    #[schemars(rename = "PounceConvexSettings")]
    pub struct Settings {
        projection: { name: "PounceConvexSettings", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
            reserved_tags: [], reserved_names: [] };
        /// Solve through the homogeneous self-dual embedding, which detects primal and dual
        /// infeasibility from the iterates, instead of the direct infeasible-start method.
        pub self_dual: bool => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Ruiz-equilibrate the data before the solve.
        pub equilibrate: bool => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Cross over from the interior-point solution to a vertex of a linear program.
        pub crossover: bool => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Native linear settings. A batch factors each instance serially: its parallelism is
        /// across instances.
        #[serde(with = "super::pounce::FeralIdentity")]
        pub linear: super::pounce::LinearSettings => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both,
            rules: [], adapter: super::pounce::FeralIdentity };
    }
}
