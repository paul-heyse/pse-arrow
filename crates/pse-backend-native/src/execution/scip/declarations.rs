// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pse_contract::projection_struct! {
/// Typed SCIP controls; identity and native admission retain their original owners.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "ScipSettings")]
pub struct Settings {
    projection: { name: "ScipSettings", rust_path: crate::execution::ScipSettings, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Linear solver of the nested Ipopt (`nlpi/ipopt/linear_solver`): the image's one
    /// Ipopt, so the same type the Ipopt adapter's settings select.
    pub nlp_linear_solver: IpoptLinearSolver => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Random seed shift (`randomization/randomseedshift`), recorded with the result.
    pub seed: u16 => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Total branch-and-bound node budget including restarts (`limits/totalnodes`);
    /// absent leaves the deadline as the only budget.
    pub nodes: Option<u32> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Number of ranked stored solutions reported beside the candidate (0: none).
    pub pool: u16 => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// After a proof of infeasibility, compute an irreducible infeasible subsystem of the
    /// true exported program (`SCIPgenerateIIS`).
    pub iis: bool => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Exact rational MILP (`SCIPenableExactSolving`); admitted for linear programs
    /// without native forms, and never with reoptimization, IIS generation (SCIP's IIS
    /// finders do not support exact solving) or concurrency.
    pub exact: bool => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Retain the native search tree across a finite MIP sequence whose constraint system
    /// is unchanged and whose linear objective changes (`SCIPenableReoptimization`).
    pub reoptimize: bool => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
