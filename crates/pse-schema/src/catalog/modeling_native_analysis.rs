// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native analysis projections retain the distinction between affine and local evidence.
use super::declarations::{column, relation, run_id};
use crate::{
    builder::RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;
fn real() -> T {
    T::native(D::Float64)
}
fn text() -> T {
    T::native(D::Utf8)
}
fn integer() -> T {
    T::native(D::Int64)
}
fn flag() -> T {
    T::native(D::Boolean)
}
fn record(fields: Vec<(&str, T)>) -> T {
    T::structure(fields.into_iter().map(|(n, t)| t.with_name(n)).collect())
}
fn coordinates() -> T {
    T::list(record(vec![("source_id", T::id()), ("value", real())]))
}
fn termination() -> T {
    record(vec![
        ("category", T::enumeration("NativeTermination")),
        ("code", integer()),
        ("name", text()),
    ])
}
fn attempt() -> T {
    record(vec![
        ("termination", termination()),
        ("qualification", T::enumeration("NativeQualification")),
        ("validation_error", text().optional()),
    ])
}
pub(super) fn register(b: &mut RegistryBuilder) {
    let range_value = || {
        record(vec![
            ("kind", T::enumeration("ModelingRealValueKind")),
            ("value", real().optional()),
        ])
    };
    let member = || T::list(record(vec![("source_id", T::id()), ("code", integer())]));
    let endpoint = || {
        record(vec![
            ("source_id", T::id().optional()),
            ("row_slack", flag()),
            ("sentinel", integer().optional()),
        ])
    };
    relation(
        b,
        N::Runtime,
        "modeling_linear_diagnostics",
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
            column("source_identity", T::hash()),
            column("numerical_identity", T::hash()),
            column("rows", T::list(T::id())),
            column("columns", T::list(T::id())),
            column("attempt", attempt()),
            column("primal_ray", coordinates()).optional(),
            column("dual_ray", coordinates()).optional(),
            column(
                "iis",
                record(vec![
                    ("columns", member()),
                    ("rows", member()),
                    ("column_status", member()),
                    ("row_status", member()),
                    ("relaxation_only", flag()),
                ]),
            )
            .optional(),
            column(
                "ranging",
                T::list(record(vec![
                    ("family", text()),
                    ("source_id", T::id()),
                    ("value", range_value()),
                    ("objective", range_value()),
                    ("entering", endpoint()),
                    ("leaving", endpoint()),
                ])),
            ),
            column(
                "relaxation",
                record(vec![
                    ("operation_status", integer()),
                    ("restored_status", termination()),
                    ("penalty", real().optional()),
                    ("primal", coordinates().optional()),
                ]),
            )
            .optional(),
            column(
                "unavailable",
                T::list(record(vec![("analysis", text()), ("reason", text())])),
            ),
        ],
        "Independent native affine-model diagnostics in physical source coordinates. IIS for a discrete model concerns its continuous relaxation. A feasibility-relaxation candidate is separate from the original attempt. Basis endpoints identify original variables or row slacks; negative native sentinels remain explicit. Infinite ranging endpoints are meaningful.",
    );
    let certificate = || {
        record(vec![
            ("weights", coordinates()),
            ("residual_maximum", real()),
            ("pivot", T::id()),
        ])
    };
    relation(
        b,
        N::Runtime,
        "modeling_jacobian_optimization",
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
            column("source_identity", T::hash()),
            column("numerical_identity", T::hash()),
            column("point", coordinates()),
            column("row_nominals", coordinates()),
            column("variable_nominals", coordinates()),
            column("tolerance", real()),
            column("rank_relative", real()),
            column("multiplier_bound", real()),
            column("complete", flag()),
            column("conditioning", T::list(certificate())),
            column(
                "degenerate",
                T::list(record(vec![
                    ("rows", T::list(T::id())),
                    ("certificate", certificate()),
                    ("irreducible_at_tolerance", flag()),
                ])),
            ),
            column("attempts", T::list(attempt())),
            column("unavailable", T::list(text())),
        ],
        "Bounded LP conditioning and minimum-support MILP evidence about the scaled Jacobian at the recorded physical point. Weights refer to rows scaled by their recorded nominals; columns are scaled by variable nominals. Numerical irreducibility at tolerance does not certify nonlinear infeasibility or symbolic rank. Attempts are ordered by source-row pivot, LP then MILP.",
    );
}
