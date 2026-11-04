// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pse_contract::projection_struct! {
/// HiGHS method, diagnostic work and partial source-attributed MIP start.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "HighsSettings")]
pub struct Settings {
    projection: { name: "HighsSettings", rust_path: crate::settings::highs::Settings, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Eligible LP algorithm; mixed models retain native class routing.
    pub method: Method => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// MIP node budget, separate from the iteration budget (F10). `None` leaves the native
    /// default (no node limit), so the time limit alone bounds the search.
    pub nodes: Option<u32> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: i32::MAX as u64 }] };
    /// Opt-in native work, separate from the original candidate except the
    /// fixed-commitment LP, which prices it.
    pub diagnostics: Request => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Partial source-attributed MIP start, with unspecified coordinates absent.
    #[schemars(with = "Option<pse_ids::document::KeyedMap<SemanticId, f64>>")]
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub sparse_start: Option<BTreeMap<SemanticId, f64>> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
}
}

pse_contract::projection_struct! {
/// Requested native diagnostics, bounded by the original attempt's deadline.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "HighsDiagnostics")]
pub struct Request {
    projection: { name: "HighsDiagnostics", rust_path: crate::settings::highs::Request, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Native primal/dual rays when available for the continuous model.
    pub rays: bool => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Native irreducible infeasible subsystem; MIP scope is its LP relaxation.
    pub iis: bool => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Basis sensitivity ranges for an optimal continuous LP with a valid basis.
    pub ranging: bool => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Separate feasibility-relaxation solve on a copied LP/MIP. Negative penalties
    /// forbid violation, as in the native API; no penalty is inferred from units.
    pub relaxation: Option<Penalties> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Duals of the MIP's LP with its discrete columns fixed at the solution
    /// (`Highs_getFixedLp`), conditional on that commitment. When that LP reaches the MIP
    /// candidate's objective they become the candidate's multipliers, and the candidate
    /// states the commitment (ADR-0118 item 9).
    pub fixed_lp: bool => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Rows of the basis inverse `B⁻¹` at these basis positions, with the basic variables,
    /// for an optimal continuous LP with a valid basis.
    pub basis_inverse: Option<Vec<usize>> => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Native presolve of a copied model: the presolved LP and, for a continuous LP, the
    /// postsolved solution of that LP.
    pub presolve: bool => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// The MIP solver's cut pool after root cut generation (callback kind 7).
    pub cut_pool: bool => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
/// Complete physical penalties for native feasibility relaxation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "HighsPenalties")]
pub struct Penalties {
    projection: { name: "HighsPenalties", rust_path: crate::settings::highs::Penalties, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Global lower-bound, upper-bound and constraint penalties.
    pub global: [FiniteBound; 3] => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Optional per-variable lower-bound penalties.
    #[serde(default)]
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub lower: Option<Vec<f64>> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Optional per-variable upper-bound penalties.
    #[serde(default)]
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub upper: Option<Vec<f64>> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Optional per-row penalties.
    #[serde(default)]
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub rows: Option<Vec<f64>> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
}
}
