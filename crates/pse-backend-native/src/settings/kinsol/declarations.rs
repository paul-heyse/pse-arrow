// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

const ETA_POWER_DOMAIN: pse_contract::boundary_projection::Float64Domain =
    pse_contract::boundary_projection::Float64Domain::Interval(
        pse_contract::boundary_projection::Float64Interval::new(1.0, false, 2.0, true),
    );
const NEWTON_STEP_DOMAIN: pse_contract::boundary_projection::Float64Domain =
    pse_contract::boundary_projection::Float64Domain::Interval(
        pse_contract::boundary_projection::Float64Interval::new(1.0, true, f64::MAX, true),
    );

pse_contract::projection_enum! {
    /// Selected native linear algebra. Dense allocation has an explicit dimension ceiling;
    /// the matrix-free Krylov routes use the analytic Jacobian-vector product.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    #[schemars(rename = "KinsolLinear")]
    pub enum Linear {
        projection: { name: "KinsolLinear", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
            reserved_tags: [], reserved_names: [] };
        /// Vendored SuiteSparse KLU with analytic CSC Jacobian.
        Klu => 1;
        /// Serial dense factorization for explicitly small systems.
        Dense {
            /// Maximum admitted dimension.
            limit: PositiveCount => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        } => 2;
        /// Matrix-free native GMRES.
        Spgmr {
            /// Maximum Krylov subspace dimension.
            dimension: PositiveCount => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        } => 3;
        /// Matrix-free native flexible GMRES.
        Spfgmr {
            /// Maximum Krylov subspace dimension.
            dimension: PositiveCount => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        } => 4;
        /// Matrix-free native BiCGStab.
        Spbcgs {
            /// Maximum Krylov subspace dimension.
            dimension: PositiveCount => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        } => 5;
        /// Matrix-free native transpose-free QMR.
        Sptfqmr {
            /// Maximum Krylov subspace dimension.
            dimension: PositiveCount => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        } => 6;
    }
}
pse_contract::projection_enum! {
    /// Inexact-Newton forcing term of the Krylov routes (`KINSetEtaForm`).
    #[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    #[schemars(rename = "KinsolEta")]
    pub enum Eta {
        projection: { name: "KinsolEta", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
            reserved_tags: [], reserved_names: [] };
        /// Eisenstat-Walker choice 1, KINSOL's default.
        #[default]
        Choice1 => 1;
        /// Eisenstat-Walker choice 2 (`KINSetEtaParams`).
        Choice2 {
            /// Safeguard factor in (0, 1].
            gamma: Fraction => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
            /// Power in (1, 2].
            #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
            alpha: f64 => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both,
                rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(ETA_POWER_DOMAIN)] };
        } => 2;
        /// A constant forcing term in (0, 1] (`KINSetEtaConstValue`).
        Constant {
            /// The forcing term.
            value: Fraction => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        } => 3;
    }
}
pse_contract::projection_struct! {
    /// Caller-selected KINSOL method controls: the adapter's pse-owned settings type.
    #[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(default, deny_unknown_fields)]
    #[schemars(rename = "KinsolSettings")]
    pub struct Method {
        projection: { name: "KinsolSettings", version: 1, direction: pse_contract::boundary_projection::Direction::Both,
            reserved_tags: [], reserved_names: [] };
        /// Nonlinear strategy.
        pub strategy: Strategy => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Native linear solver for equation/Picard profiles; fixed point requires the default.
        pub linear: Linear => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Native Anderson history for Picard/fixed point; zero disables acceleration.
        pub anderson: usize => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Picard/fixed-point damping in (0,1]; Newton and line search require one.
        pub damping: Fraction => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Maximum nonlinear iterations between linear setups; fixed point requires the default.
        pub setup_interval: u32 => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both,
            rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: u32::MAX as u64 }] };
        /// Maximum scaled Newton step (`KINSetMaxNewtonStep`), at least one scaled unit
        /// because KINSOL raises a smaller cap to one; Newton/line search only. `None` keeps KINSOL's
        /// `1000 * ||D_u u_0||`.
        #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
        pub max_newton_step: Option<f64> => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both,
            rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(NEWTON_STEP_DOMAIN)] };
        /// Inexact-Newton forcing term; Krylov routes only.
        pub eta: Eta => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Krylov preconditioner from the analytic Jacobian (`KINSetPreconditioner`);
        /// KINSOL preconditions on the right.
        pub preconditioner: Preconditioner => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Anderson QR orthogonalization; Anderson acceleration only.
        pub orthogonalization: Orthogonalization => { tag: 9, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Iterations before Anderson acceleration starts (`KINSetDelayAA`); Anderson only.
        pub anderson_delay: usize => { tag: 10, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    }
}
