// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The HiGHS settings document: the LP method, the MIP node budget, opt-in native
//! diagnostics and a partial MIP start.
use pse_ids::SemanticId;
use pse_model::scalars::FiniteBound;
use std::collections::BTreeMap;

/// Explicit native LP method, a registry vocabulary (ADR-0115 Outcome 3); automatic
/// remains a native class-specific decision.
pub use pse_model::generated::enums::HighsMethod as Method;

/// The HiGHS adapter's settings type on the unified lifecycle; identity derives from serde,
/// and absent fields take these defaults across the Python boundary (ADR-0113).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "HighsSettings")]
pub struct Settings {
    /// Eligible LP algorithm; mixed models retain native class routing.
    pub method: Method,
    /// MIP node budget, separate from the iteration budget (F10). `None` leaves the native
    /// default (no node limit), so the time limit alone bounds the search.
    pub nodes: Option<u32>,
    /// Opt-in native work, separate from the original candidate.
    pub diagnostics: Request,
    /// Partial source-attributed MIP start, with unspecified coordinates absent.
    pub sparse_start: Option<BTreeMap<SemanticId, f64>>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            method: Method::Choose,
            nodes: None,
            diagnostics: Request::default(),
            sparse_start: None,
        }
    }
}

/// Requested native diagnostic work, bounded by the original attempt's deadline.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "HighsDiagnostics")]
pub struct Request {
    /// Native primal/dual rays when available for the continuous model.
    pub rays: bool,
    /// Native irreducible infeasible subsystem; MIP scope is its LP relaxation.
    pub iis: bool,
    /// Basis sensitivity ranges for an optimal continuous LP with a valid basis.
    pub ranging: bool,
    /// Separate feasibility-relaxation solve on a copied LP/MIP. Negative penalties
    /// forbid violation, as in the native API; no penalty is inferred from units.
    pub relaxation: Option<Penalties>,
    /// Duals of the MIP's LP with its discrete columns fixed at the solution
    /// (`Highs_getFixedLp`), conditional on that commitment.
    pub fixed_lp: bool,
    /// Rows of the basis inverse `B⁻¹` at these basis positions, with the basic variables,
    /// for an optimal continuous LP with a valid basis.
    pub basis_inverse: Option<Vec<usize>>,
    /// Native presolve of a copied model: the presolved LP and, for a continuous LP, the
    /// postsolved solution of that LP.
    pub presolve: bool,
    /// The MIP solver's cut pool after root cut generation (callback kind 7).
    pub cut_pool: bool,
}
impl Request {
    /// Any diagnostic work was requested.
    pub fn any(&self) -> bool {
        self.rays
            || self.iis
            || self.ranging
            || self.relaxation.is_some()
            || self.fixed_lp
            || self.basis_inverse.is_some()
            || self.presolve
            || self.cut_pool
    }
}
/// Complete physical penalty declarations for native feasibility relaxation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "HighsPenalties")]
pub struct Penalties {
    /// Global lower-bound, upper-bound and constraint penalties.
    pub global: [FiniteBound; 3],
    /// Optional per-variable lower-bound penalties.
    pub lower: Option<Vec<f64>>,
    /// Optional per-variable upper-bound penalties.
    pub upper: Option<Vec<f64>>,
    /// Optional per-row penalties.
    pub rows: Option<Vec<f64>>,
}
