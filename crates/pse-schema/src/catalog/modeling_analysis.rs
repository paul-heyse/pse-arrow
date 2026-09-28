// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned analysis evidence; source identities remain distinct from algorithm coordinates.
use super::declarations::{column, enumeration, identity, relation, relation_version, run_id};
use crate::{
    builder::RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;
fn text() -> T {
    T::native(D::Utf8)
}
fn real() -> T {
    T::native(D::Float64)
}
fn count() -> T {
    T::nonnegative(i64::MAX)
}
fn flag() -> T {
    T::native(D::Boolean)
}
fn record(fields: Vec<(&str, T)>) -> T {
    T::structure(
        fields
            .into_iter()
            .map(|(name, ty)| ty.with_name(name))
            .collect(),
    )
}
fn coordinates() -> T {
    T::list(record(vec![("source_id", T::id()), ("value", real())]))
}
pub(super) fn register(b: &mut RegistryBuilder) {
    identity(
        b,
        "instance",
        "One instantiated definition instance of a specialized model. A root instance takes its root declaration's identity unless it is a fit experiment, which prepares its case under the experiment's own identity; a nested instance is derived from its parent and member",
    );
    enumeration(
        b,
        "ModelingRealValueKind",
        [
            "finite",
            "negative_infinity",
            "positive_infinity",
            "indeterminate",
        ],
    );
    enumeration(
        b,
        "ModelingDiagnosticSampleStop",
        [
            "completed",
            "sample_limit",
            "finding_limit",
            "time_limit",
            "cancelled",
        ],
    );
    relation(
        b,
        N::Runtime,
        "modeling_diagnostic_samples",
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
            column("unattempted", count()),
            column("stop", T::enumeration("ModelingDiagnosticSampleStop")),
            column(
                "outcomes",
                T::list(record(vec![
                    ("sample_id", T::id()),
                    ("report_id", T::id().with_identity("run").optional()),
                    ("failure_ordinal", count().optional()),
                    (
                        "error_class",
                        T::enumeration("NativeBoundaryClass").optional(),
                    ),
                    ("error", text().optional()),
                ])),
            ),
        ],
        "Named bounded diagnostic samples link to individually owned reports. Evaluation or binding failures retain their sample identity and classification; unattempted samples are not failed attempts.",
    );
    let parallel = || {
        T::list(record(vec![
            ("first_id", T::id()),
            ("second_id", T::id()),
            ("cosine", real()),
        ]))
    };
    relation(
        b,
        N::Runtime,
        "modeling_diagnostics",
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
            column("source_identity", T::hash()),
            column("numerical_identity", T::hash()),
            column("profile", text()),
            column("complete", flag()),
            column(
                "point",
                T::list(record(vec![
                    ("source_id", T::id()),
                    ("kind", T::enumeration("ModelingRealValueKind")),
                    ("value", real().optional()),
                ])),
            ),
            column("rows", T::list(T::id())),
            column("columns", T::list(T::id())),
            column("row_nominals", coordinates()),
            column("variable_nominals", coordinates()),
            column(
                "statistics",
                T::list(record(vec![("name", text()), ("count", count())])),
            ),
            column(
                "matrix",
                record(vec![
                    ("rank", count()),
                    ("cutoff", real()),
                    ("row_norms", coordinates()),
                    ("column_norms", coordinates()),
                    ("parallel_rows", parallel()),
                    ("parallel_columns", parallel()),
                    (
                        "modes",
                        T::list(record(vec![
                            ("value", real()),
                            ("left", coordinates()),
                            ("right", coordinates()),
                        ])),
                    ),
                ]),
            )
            .optional(),
        ],
        "Numerical diagnostics at an explicit physical point. Matrix vectors name semantic rows and variables, including rectangular null modes. Completeness concerns the requested bounded analyses, not structural rank or feasibility.",
    );
    relation_version(
        b,
        N::Runtime,
        "modeling_findings",
        2,
        S::Derived,
        &["run_id", "ordinal"],
        vec![
            run_id(),
            column("ordinal", count()),
            column("class", T::enumeration("NativeBoundaryClass")),
            column("severity", T::enumeration("DiagnosticSeverity")),
            column("stage", text()),
            column("rule", text()),
            column("sources", T::list(T::id())),
            column(
                "observations",
                T::list(record(vec![
                    ("name", text()),
                    ("kind", T::enumeration("NativeMetricKind")),
                    ("real", real().optional()),
                    (
                        "real_kind",
                        T::enumeration("ModelingRealValueKind").optional(),
                    ),
                    ("integer", T::native(D::Int64).optional()),
                    ("boolean", flag().optional()),
                    ("text", text().optional()),
                ])),
            ),
            column(
                "locations",
                T::list(record(vec![
                    ("source_id", T::id()),
                    ("path", text()),
                    ("name", text().optional()),
                    ("start", count().optional()),
                    ("end", count().optional()),
                ])),
            ),
        ],
        "Attributed diagnostic findings with a class and a severity; a warning is never an invalid model. The observation kind selects its payload; real_kind classifies finite, infinite and indeterminate values, with a numeric real payload only when finite. Absent values are not zero. Locations describe source declarations rather than native matrix indices.",
    );
    enumeration(
        b,
        "ModelingInitializationStep",
        ["stage", "homotopy", "original"],
    );
    relation(
        b,
        N::Runtime,
        "modeling_initializations",
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
            column("complete", flag()),
            column("failure", text()).optional(),
            column("failure_ordinal", count()).optional(),
            column("committed", coordinates()).optional(),
            column(
                "attempts",
                T::list(record(vec![
                    ("kind", T::enumeration("ModelingInitializationStep")),
                    ("stage", text().optional()),
                    ("fraction", real().optional()),
                    ("result_id", T::id().with_identity("run").optional()),
                    ("accepted", flag()),
                    ("error", text().optional()),
                    ("failure_ordinal", count().optional()),
                    (
                        "interruption_class",
                        T::enumeration("NativeBoundaryClass").optional(),
                    ),
                    ("interruption", text().optional()),
                ])),
            ),
        ],
        "Ordered immutable initialization attempts. Only an accepted original specification supplies committed values. Native result IDs link separately owned original-space result tables.",
    );
    relation(
        b,
        N::Runtime,
        "modeling_studies",
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
            column("unattempted", count()),
            column(
                "points",
                T::list(record(vec![
                    ("root_id", T::id().with_identity("declaration").optional()),
                    ("instance_id", T::id().with_identity("instance").optional()),
                    ("predecessor", count().optional()),
                    ("result_id", T::id().with_identity("run").optional()),
                    ("accepted", flag()),
                    ("error", text().optional()),
                    ("failure_ordinal", count().optional()),
                ])),
            ),
        ],
        "Ordered study outcomes including declaration preparation failures and explicit accepted-predecessor dependencies. Unattempted points remain distinct from failed attempts.",
    );
    // The summary a durable study publishes (Plan 22 O7), one row per point, beside every
    // completed point's result members.
    b.declare_relation(
        super::declarations::declaration(
            N::Runtime,
            "study_outcomes",
            1,
            S::Derived,
            &["study_id", "point_index"],
            vec![
                column("study_id", T::id()).with_identity("study"),
                column("point_index", count()),
                column("case_id", T::id()).with_identity("declaration"),
                column("binding_hash", T::hash()),
                column("predecessor", count()).optional(),
                column("state", T::enumeration("StudyPointState")),
                column("attempt_id", T::id()).with_identity("attempt"),
                column("attempt_state", T::enumeration("AttemptState")),
                column("member_catalog", text()).optional(),
                column("error", text()).optional(),
            ],
            "The outcome of every point of a durable study, published once with the study: its authored case and value bindings by hash, the earlier point that seeded it, its final state, the attempt that ended it and that attempt's state. A completed point's result members are published under `member_catalog`; a failed or cancelled point contributes no members and records why in `error`.",
        )
        .check(
            "members_of_completed_points",
            "(\"state\" = 'completed') = (\"member_catalog\" IS NOT NULL)",
        )
        .check(
            "predecessor_is_earlier",
            "\"predecessor\" IS NULL OR \"predecessor\" < \"point_index\"",
        ),
    );
    enumeration(
        b,
        "ModelingElasticObservation",
        ["feasible_witness", "local_obstruction", "inconclusive"],
    );
    relation(
        b,
        N::Runtime,
        "modeling_nonlinear_explanations",
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
            column("source_identity", T::hash()),
            column("complete", flag()),
            column("stop", text()).optional(),
            column("failure_ordinal", count()).optional(),
            column("candidate_rows", T::list(T::id())),
            column("background_variables", T::list(T::id())),
            column("nominals", coordinates()),
            column("penalty_tolerance", real()),
            column(
                "attempts",
                T::list(record(vec![
                    ("omitted", T::list(T::id())),
                    ("observation", T::enumeration("ModelingElasticObservation")),
                    ("penalty", real().optional()),
                    ("result_id", T::id().with_identity("run").optional()),
                    ("failure_ordinal", count().optional()),
                    ("error", text().optional()),
                    (
                        "interruption_class",
                        T::enumeration("NativeBoundaryClass").optional(),
                    ),
                    ("interruption", text().optional()),
                ])),
            ),
        ],
        "Bounded elastic deletion evidence at explicit physical weights under unchanged bounds and inner systems. Positive slack at a local stationary solution does not certify infeasibility; complete never asserts global minimality.",
    );
    relation(
        b,
        N::Runtime,
        "modeling_trajectory_modes",
        S::Derived,
        &["run_id", "sample"],
        vec![
            run_id(),
            column("sample", count()),
            column("time", real()),
            column("mode", text()),
        ],
        "Authored analysis mode name at each completed trajectory sample, after coincident nonterminal resets. Native mode indices are not persisted.",
    );
}
