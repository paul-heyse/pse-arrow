// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Serde JSON records of POUNCE's warm-start and crossover reports, never Rust `Debug` text
//! (F30). Each record names every field of its library type through an exhaustive
//! destructuring, so a POUNCE upgrade that adds a field fails to compile instead of dropping
//! it from the published provenance; enumerations take snake_case names.
use pounce_rs::pounce_algorithm::{
    crossover::{CrossoverDecline, CrossoverPhase, CrossoverReport},
    init::warm_start::{BlockVerdict, WarmStartDiagnostics},
};
use serde_json::json;

const fn verdict(verdict: BlockVerdict) -> &'static str {
    match verdict {
        BlockVerdict::Absent => "absent",
        BlockVerdict::Accepted => "accepted",
        BlockVerdict::Reconstructed => "reconstructed",
        BlockVerdict::Discarded => "discarded",
        BlockVerdict::Rejected => "rejected",
        BlockVerdict::Unseeded => "unseeded",
    }
}

/// What the interior-point warm start did with the submitted primal-dual seed.
pub(super) fn warm(diagnostics: &WarmStartDiagnostics) -> String {
    let WarmStartDiagnostics {
        primal_residual,
        dual_residual,
        complementarity,
        mu_in,
        mu_out,
        bound_duals,
        eq_duals,
        bound_duals_reconstructed,
        bound_duals_rejected,
        eq_duals_rejected,
        stationarity_split,
        recentering_disabled,
    } = diagnostics;
    json!({
        "primal_residual": primal_residual,
        "dual_residual": dual_residual,
        "complementarity": complementarity,
        "mu_in": mu_in,
        "mu_out": mu_out,
        "bound_duals": verdict(*bound_duals),
        "eq_duals": verdict(*eq_duals),
        "bound_duals_reconstructed": bound_duals_reconstructed,
        "bound_duals_rejected": bound_duals_rejected,
        "eq_duals_rejected": eq_duals_rejected,
        "stationarity_split": stationarity_split,
        "recentering_disabled": recentering_disabled,
    })
    .to_string()
}

/// What crossover did: the phase that produced the accepted point, or why it declined.
pub(super) fn crossover(report: &CrossoverReport) -> String {
    let CrossoverReport {
        phase,
        declined,
        n_iter,
        n_qp_solves,
        active_bounds,
        active_constraints,
        estimated_active,
        kkt_before,
        kkt_after,
        compl_after,
    } = report;
    let phase = phase.map(|phase| match phase {
        CrossoverPhase::EqpStep => "eqp_step",
        CrossoverPhase::ActiveSet => "active_set",
    });
    let declined = declined.map(|reason| match reason {
        CrossoverDecline::NothingToIdentify => "nothing_to_identify",
        CrossoverDecline::QpFailed => "qp_failed",
        CrossoverDecline::LineSearchFailed => "line_search_failed",
        CrossoverDecline::ActiveSetNotConverged => "active_set_not_converged",
        CrossoverDecline::Regressed => "regressed",
    });
    json!({
        "phase": phase,
        "declined": declined,
        "n_iter": n_iter,
        "n_qp_solves": n_qp_solves,
        "active_bounds": active_bounds,
        "active_constraints": active_constraints,
        "estimated_active": estimated_active,
        "kkt_before": kkt_before,
        "kkt_after": kkt_after,
        "compl_after": compl_after,
    })
    .to_string()
}
