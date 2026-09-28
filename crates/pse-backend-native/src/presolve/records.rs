// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Serde JSON records of the library's presolve reports, never Rust `Debug` text (F30).
//! Each record names every field of its library type through an exhaustive destructuring,
//! so a pounce-presolve upgrade that adds a field fails to compile instead of dropping it
//! from the published report; enumerations take snake_case names.
use pounce_presolve::{
    AuxiliaryPreprocessingDiagnostics, AuxiliaryRejectionReason, LicqVerdict, LinearEqElimReport,
    TightenReport, WarmProjectionReport,
    diagnostics::{ClassCounts, StageTimings},
    fbbt::FbbtReport,
};
use serde_json::{Value, json};

/// Milliseconds as a JSON integer; a `u128` duration beyond `u64` saturates.
fn millis(ms: u128) -> u64 {
    u64::try_from(ms).unwrap_or(u64::MAX)
}

/// Linear bound tightening.
pub(super) fn tighten(report: &TightenReport) -> String {
    let TightenReport {
        n_tightened,
        n_new_finite,
        infeasible,
    } = report;
    json!({
        "n_tightened": n_tightened,
        "n_new_finite": n_new_finite,
        "infeasible": infeasible,
    })
    .to_string()
}

/// Feasibility-based bound tightening; `null` when it did not run.
pub(super) fn fbbt(report: Option<&FbbtReport>) -> String {
    report
        .map_or(Value::Null, |report| {
            let FbbtReport {
                iterations,
                bound_updates,
                infeasibility_witness,
                total_tightening,
            } = report;
            json!({
                "iterations": iterations,
                "bound_updates": bound_updates,
                "infeasibility_witness": infeasibility_witness,
                "total_tightening": total_tightening,
            })
        })
        .to_string()
}

/// The structural LICQ verdict in serde's externally tagged form; `null` when unchecked.
pub(super) fn licq(verdict: Option<&LicqVerdict>) -> String {
    verdict
        .map_or(Value::Null, |verdict| match verdict {
            LicqVerdict::Full => json!("full"),
            LicqVerdict::EmptyRow(row) => json!({"empty_row": row}),
            LicqVerdict::OverDetermined { m_eq, n } => {
                json!({"over_determined": {"m_eq": m_eq, "n": n}})
            }
            LicqVerdict::StructuralRank(rank) => json!({"structural_rank": rank}),
        })
        .to_string()
}

const fn rejection(reason: &AuxiliaryRejectionReason) -> &'static str {
    match reason {
        AuxiliaryRejectionReason::BlockTooLarge => "block_too_large",
        AuxiliaryRejectionReason::CouplingDisallowed => "coupling_disallowed",
        AuxiliaryRejectionReason::BlockSolveDiverged => "block_solve_diverged",
        AuxiliaryRejectionReason::ResidualCheckFailed => "residual_check_failed",
        AuxiliaryRejectionReason::OutOfBounds => "out_of_bounds",
        AuxiliaryRejectionReason::NonBlockColumnFree => "non_block_column_free",
    }
}

/// Auxiliary (Phase 0) block elimination.
pub(super) fn auxiliary(diagnostics: &AuxiliaryPreprocessingDiagnostics) -> String {
    let AuxiliaryPreprocessingDiagnostics {
        blocks_eliminated,
        candidate_blocks,
        vars_eliminated,
        rows_eliminated,
        total_time_ms,
        stage_time_ms,
        class_counts,
        max_block_residual,
        max_accepted_block_dim,
        rejection_reasons,
        trivially_fixed_vars,
        trivially_free_rows,
        trivially_slack_rows,
        inequality_coupled_accepted_via_projection,
    } = diagnostics;
    let StageTimings {
        incidence_ms,
        matching_ms,
        dm_ms,
        components_ms,
        btf_ms,
        block_solve_ms,
        residual_check_ms,
    } = stage_time_ms;
    let ClassCounts {
        pure_equality,
        objective_coupled,
        inequality_coupled,
        objective_and_inequality_coupled,
    } = class_counts;
    json!({
        "blocks_eliminated": blocks_eliminated,
        "candidate_blocks": candidate_blocks,
        "vars_eliminated": vars_eliminated,
        "rows_eliminated": rows_eliminated,
        "total_time_ms": millis(*total_time_ms),
        "stage_time_ms": {
            "incidence_ms": millis(*incidence_ms),
            "matching_ms": millis(*matching_ms),
            "dm_ms": millis(*dm_ms),
            "components_ms": millis(*components_ms),
            "btf_ms": millis(*btf_ms),
            "block_solve_ms": millis(*block_solve_ms),
            "residual_check_ms": millis(*residual_check_ms),
        },
        "class_counts": {
            "pure_equality": pure_equality,
            "objective_coupled": objective_coupled,
            "inequality_coupled": inequality_coupled,
            "objective_and_inequality_coupled": objective_and_inequality_coupled,
        },
        "max_block_residual": max_block_residual,
        "max_accepted_block_dim": max_accepted_block_dim,
        "rejection_reasons": rejection_reasons.iter().map(rejection).collect::<Vec<_>>(),
        "trivially_fixed_vars": trivially_fixed_vars,
        "trivially_free_rows": trivially_free_rows,
        "trivially_slack_rows": trivially_slack_rows,
        "inequality_coupled_accepted_via_projection": inequality_coupled_accepted_via_projection,
    })
    .to_string()
}

/// Affine linear-equality elimination.
pub(super) fn elimination(report: &LinearEqElimReport) -> String {
    let LinearEqElimReport {
        n_constant_vars,
        n_aggregated_vars,
        n_rows_eliminated,
        n_redundant_rows,
        passes,
        pass_cap_hit,
        infeasible,
    } = report;
    json!({
        "n_constant_vars": n_constant_vars,
        "n_aggregated_vars": n_aggregated_vars,
        "n_rows_eliminated": n_rows_eliminated,
        "n_redundant_rows": n_redundant_rows,
        "passes": passes,
        "pass_cap_hit": pass_cap_hit,
        "infeasible": infeasible,
    })
    .to_string()
}

/// Projection of a warm start into the reduced problem.
pub(super) fn projection(report: &WarmProjectionReport) -> String {
    let WarmProjectionReport {
        n_dropped_rows,
        dropped_dual_l1,
        x_clamped_count,
        x_fixed_overridden_count,
    } = report;
    json!({
        "n_dropped_rows": n_dropped_rows,
        "dropped_dual_l1": dropped_dual_l1,
        "x_clamped_count": x_clamped_count,
        "x_fixed_overridden_count": x_fixed_overridden_count,
    })
    .to_string()
}
