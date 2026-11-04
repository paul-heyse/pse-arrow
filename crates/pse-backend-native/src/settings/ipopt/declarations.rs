// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

const MUMPS_PIVOT_DOMAIN: pse_contract::boundary_projection::Float64Domain =
    pse_contract::boundary_projection::Float64Domain::Interval(
        pse_contract::boundary_projection::Float64Interval::new(0.0, true, 1.0, true),
    );

pse_contract::projection_struct! {
    /// The Ipopt adapter's settings type. Its identity derives from serde, and absent fields
    /// take these defaults across the Python boundary (ADR-0113).
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(default, deny_unknown_fields)]
    #[schemars(rename = "IpoptSettings")]
    pub struct Settings {
        projection: { name: "IpoptSettings", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
            reserved_tags: [], reserved_names: [] };
        /// Symmetric-indefinite factorization of the KKT systems.
        pub linear: Linear => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Barrier-parameter update strategy (`mu_strategy`).
        pub mu_strategy: MuStrategy => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Minimum absolute distance of a cold initial point from its bounds (`bound_push`).
        pub bound_push: Tolerance => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Minimum relative distance of a cold initial point from its bounds (`bound_frac`);
        /// admission refuses more than one half, Ipopt's registered range.
        pub bound_frac: Fraction => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both,
            rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(crate::solve::RELATIVE_PUSH_DOMAIN)] };
        /// Restart of a submitted primal-dual seed: barrier and pushes (L-N3).
        pub restart: WarmRestart => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    }
}
pse_contract::projection_enum! {
    /// One linked symmetric-indefinite factorization with its typed parameters. HSL solvers and
    /// the runtime-loaded Pardiso are not representable (ADR-0108 item 9). Each kind is spelled
    /// as its [`IpoptLinearSolver`], Ipopt's native `linear_solver` name.
    #[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
    #[schemars(rename = "IpoptLinear")]
    pub enum Linear {
        projection: { name: "IpoptLinear", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
            reserved_tags: [], reserved_names: [] };
        /// Sequential MUMPS 5.9 (`linear_solver = mumps`).
        Mumps {
            /// Fill-reducing ordering (`mumps_pivot_order`, MUMPS ICNTL(7)).
            ordering: MumpsOrdering => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
            /// Sparsity/stability pivot tolerance (`mumps_pivtol`), inclusive `[0,1]`.
            #[schemars(extend("minimum" = 0.0, "maximum" = 1.0))]
            #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
            pivot_tolerance: f64 => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both,
                rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(MUMPS_PIVOT_DOMAIN)] };
            /// Maximum refinement-requested pivot tolerance (`mumps_pivtolmax`).
            #[schemars(extend("minimum" = 0.0, "maximum" = 1.0))]
            #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
            maximum_pivot_tolerance: f64 => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both,
                rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(MUMPS_PIVOT_DOMAIN)] };
            /// Extra estimated workspace percentage (`mumps_mem_percent`), zero allowed.
            #[schemars(extend("minimum" = 0))]
            memory_percent: i32 => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both,
                rules: [pse_contract::boundary_projection::LocalRule::SignedRange { min: 0, max: i32::MAX as i64 }] };
            /// Native ICNTL(6) permuting/scaling strategy, inclusive `[0,7]`.
            #[schemars(extend("maximum" = 7))]
            permuting_scaling: u8 => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both,
                rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 0, max: 7 }] };
        } => 1;
        /// SPRAL SSIDS on OpenMP threads (`linear_solver = spral`).
        Spral {
            /// Elimination ordering (`spral_order`).
            ordering: SpralOrdering => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
            /// Matrix scaling (`spral_scaling`).
            scaling: SpralScaling => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
            /// A posteriori pivoting strategy (`spral_pivot_method`).
            pivot: SpralPivot => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        } => 2;
        /// oneMKL Pardiso on MKL threads (`linear_solver = pardisomkl`), under the pinned CBWR
        /// branch.
        PardisoMkl {
            /// Fill-reducing ordering (`pardisomkl_order`).
            ordering: PardisoOrdering => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
            /// Symmetric weighted matching (`pardisomkl_matching_strategy`).
            matching: PardisoMatching => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        } => 3;
    }
}
