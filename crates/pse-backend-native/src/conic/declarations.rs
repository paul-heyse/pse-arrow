// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pse_contract::projection_struct! {
/// Clarabel's pse-owned admitted controls; native defaults remain library-owned.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "ClarabelSettings")]
pub struct Settings {
    projection: { name: "ClarabelSettings", rust_path: crate::conic::Settings, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Preprocessing or data-update mode.
    pub mode: Mode => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// KKT direct solver; only MKL Pardiso admits more than one thread.
    pub direct: Direct => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Maximum interior step fraction.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub max_step_fraction: f64 => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Absolute infeasibility tolerance.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub tol_infeas_abs: f64 => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Relative infeasibility tolerance.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub tol_infeas_rel: f64 => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// KKT ratio tolerance.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub tol_ktratio: f64 => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Reduced absolute infeasibility tolerance.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub reduced_tol_infeas_abs: f64 => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Reduced relative infeasibility tolerance.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub reduced_tol_infeas_rel: f64 => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Reduced KKT ratio tolerance.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub reduced_tol_ktratio: f64 => { tag: 9, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Equilibration iterations.
    pub equilibrate_max_iter: u32 => { tag: 10, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Minimum equilibration scaling.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub equilibrate_min_scaling: f64 => { tag: 11, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Maximum equilibration scaling.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub equilibrate_max_scaling: f64 => { tag: 12, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Line-search backtracking factor.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub linesearch_backtrack_step: f64 => { tag: 13, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Minimum step length before switching to symmetric scaling.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub min_switch_step_length: f64 => { tag: 14, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Minimum step length before termination.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub min_terminate_step_length: f64 => { tag: 15, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Static KKT regularization.
    pub static_regularization_enable: bool => { tag: 16, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Constant static regularization.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub static_regularization_constant: f64 => { tag: 17, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Proportional static regularization.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub static_regularization_proportional: f64 => { tag: 18, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Dynamic KKT regularization.
    pub dynamic_regularization_enable: bool => { tag: 19, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Dynamic regularization threshold.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub dynamic_regularization_eps: f64 => { tag: 20, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Dynamic regularization shift.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub dynamic_regularization_delta: f64 => { tag: 21, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// KKT iterative refinement.
    pub iterative_refinement_enable: bool => { tag: 22, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Refinement relative tolerance.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub iterative_refinement_reltol: f64 => { tag: 23, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Refinement absolute tolerance.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub iterative_refinement_abstol: f64 => { tag: 24, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Refinement iterations.
    pub iterative_refinement_max_iter: u32 => { tag: 25, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Refinement stall ratio.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub iterative_refinement_stop_ratio: f64 => { tag: 26, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Native presolve; `ReusableData` disables it.
    pub presolve_enable: bool => { tag: 27, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Drop explicit input zeros; `ReusableData` disables it.
    pub input_sparse_dropzeros: bool => { tag: 28, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Chordal decomposition of PSD cones; `ReusableData` disables it. Without the SDP
    /// profile only the chordal defaults are admitted.
    pub chordal_decomposition_enable: bool => { tag: 29, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Clique merging.
    pub chordal_decomposition_merge_method: MergeMethod => { tag: 30, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Compact the decomposed problem.
    pub chordal_decomposition_compact: bool => { tag: 31, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Complete the dual of decomposed PSD cones.
    pub chordal_decomposition_complete_dual: bool => { tag: 32, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
