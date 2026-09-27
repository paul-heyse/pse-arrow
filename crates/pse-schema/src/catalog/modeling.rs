// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generic modeling-language declarations; the sole durable kernel IR schema.
use super::declarations::{column, enumeration, relation};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S, TaggedAlternative};
use arrow_schema::DataType as D;

fn text(name: &str) -> T {
    T::native(D::Utf8).with_name(name)
}
fn strings(name: &str) -> T {
    T::list(T::native(D::Utf8)).with_name(name)
}
fn flag(name: &str) -> T {
    T::native(D::Boolean).with_name(name)
}
fn parameters(name: &str) -> T {
    T::list(T::structure(vec![
        text("name"),
        text("type_name"),
        text("default_value").optional(),
    ]))
    .with_name(name)
}
fn indices() -> T {
    T::list(T::structure(vec![text("name"), text("domain")])).with_name("indices")
}
fn assignments(name: &str) -> T {
    T::list(T::structure(vec![text("name"), text("expression")])).with_name(name)
}

pub(super) fn declare(builder: &mut RegistryBuilder) {
    let arms = vec![
        (
            "relaxation",
            vec!["relaxation"],
            vec![text("target"), text("nominal")],
        ),
        (
            "continuation",
            vec!["continuation"],
            vec![text("target"), text("start"), text("end")],
        ),
        (
            "scope",
            vec![
                "package",
                "entity_kind",
                "interface",
                "definition",
                "case",
                "test",
                "stage",
                "implicit",
                "regime",
            ],
            vec![
                parameters("parameters"),
                strings("bases"),
                strings("type_parameters"),
                T::structure(vec![text("criterion"), text("tolerance")])
                    .with_name("selection")
                    .optional(),
                text("eligibility").optional(),
                T::structure(vec![text("reference"), text("revision")])
                    .with_name("oracle")
                    .optional(),
                T::structure(vec![
                    T::native(D::Int64).with_name("degrees_of_freedom"),
                    T::enumeration("ModelingFixtureExecution")
                        .with_name("execution")
                        .optional(),
                    strings("stages"),
                    T::structure(vec![
                        T::native(D::Boolean).with_name("homotopy"),
                        T::native(D::Float64).with_name("initial_step"),
                        T::native(D::Float64).with_name("minimum_step"),
                        T::native(D::Float64).with_name("growth"),
                        T::nonnegative(100_000).with_name("maximum_attempts"),
                        T::native(D::Float64).with_name("time_limit_seconds"),
                    ])
                    .with_name("initialization")
                    .optional(),
                    T::structure(vec![
                        strings("samples"),
                        T::native(D::Float64).with_name("relative_tolerance"),
                        T::native(D::Float64).with_name("normalized_absolute_tolerance"),
                        text("initial_step"),
                        T::native(D::Float64)
                            .with_name("quadrature_relative_tolerance")
                            .optional(),
                        T::list(T::structure(vec![
                            text("target"),
                            text("absolute_tolerance"),
                        ]))
                        .with_name("quadratures"),
                    ])
                    .with_name("integration")
                    .optional(),
                    T::structure(vec![
                        T::enumeration("NativeBoundaryClass").with_name("class"),
                        text("rule"),
                    ])
                    .with_name("expected_failure")
                    .optional(),
                    T::list(T::structure(vec![
                        text("target"),
                        T::enumeration("ModelingFixtureBinding").with_name("kind"),
                        text("expression").optional(),
                    ]))
                    .with_name("specifications"),
                ])
                .with_name("fixture")
                .optional(),
            ],
        ),
        (
            "binding",
            vec![
                "parameter",
                "variable",
                "let",
                "alias",
                "attribute",
                "set",
                "child",
                "port",
                "preset",
                "scope_value",
            ],
            vec![
                text("type_name"),
                indices(),
                text("expression").optional(),
                text("defined_by").optional(),
            ],
        ),
        (
            "function",
            vec!["function"],
            vec![
                strings("type_parameters"),
                parameters("arguments"),
                text("return_type"),
                text("body").optional(),
                text("validity").optional(),
                T::nonnegative(2).with_name("continuity").optional(),
                T::structure(vec![
                    text("implementation"),
                    text("revision"),
                    text("data"),
                    text("output"),
                    T::enumeration("ExternalDerivativeSource").with_name("derivative_source"),
                    T::nonnegative(2).with_name("derivatives"),
                    T::nonnegative(2).with_name("smoothness"),
                ])
                .with_name("external")
                .optional(),
            ],
        ),
        (
            "equation",
            vec!["equation"],
            vec![indices(), text("expression")],
        ),
        (
            "table",
            vec!["table"],
            vec![
                parameters("keys"),
                parameters("columns"),
                text("value_type"),
                text("missing_policy"),
                text("default_value").optional(),
            ],
        ),
        (
            "dataset",
            vec!["dataset"],
            vec![
                text("table"),
                text("source"),
                T::list(T::structure(vec![strings("keys"), strings("values")])).with_name("rows"),
            ],
        ),
        (
            "entity",
            vec!["entity"],
            vec![text("kind_name"), assignments("attributes")],
        ),
        ("enumeration", vec!["enum"], vec![strings("members")]),
        (
            "import",
            vec!["import"],
            vec![text("version"), text("alias").optional()],
        ),
        ("guard", vec!["when"], vec![text("predicate")]),
        (
            "accumulator",
            vec!["accumulator"],
            vec![
                indices(),
                text("type_name"),
                T::enumeration("ModelingAccumulatorMode").with_name("mode"),
                text("tolerance"),
            ],
        ),
        (
            "contribution",
            vec!["contribution"],
            vec![
                indices(),
                text("target"),
                text("expression"),
                T::enumeration("ModelingContributionRole").with_name("role"),
                text("transfer_id").optional(),
                text("transfer_side").optional(),
            ],
        ),
        (
            "connection",
            vec!["connection"],
            vec![text("from"), text("to")],
        ),
        (
            "annotation",
            vec!["annotation"],
            vec![
                text("annotation_type"),
                text("target"),
                strings("arguments"),
            ],
        ),
        (
            "requirement",
            vec!["requirement"],
            vec![text("predicate"), text("message")],
        ),
        (
            "expectation",
            vec!["expectation"],
            vec![
                text("actual"),
                text("expected"),
                text("tolerance"),
                text("relative_tolerance").optional(),
            ],
        ),
        (
            "continuous",
            vec!["continuous"],
            vec![text("type_name"), text("lower"), text("upper")],
        ),
        (
            "difference_scheme",
            vec!["difference_scheme"],
            vec![
                T::nonnegative(64).with_name("order"),
                T::list(T::native(D::Int64)).with_name("offsets"),
                T::list(T::native(D::Float64)).with_name("weights"),
                T::list(T::native(D::Float64)).with_name("quadrature"),
            ],
        ),
        (
            "collocation_scheme",
            vec!["collocation_scheme"],
            vec![
                T::native(D::Float64).with_name("alpha"),
                T::native(D::Float64).with_name("beta"),
                flag("right_endpoint"),
            ],
        ),
        (
            "discretization",
            vec!["discretization"],
            vec![
                text("target"),
                text("scheme"),
                text("elements"),
                text("order"),
            ],
        ),
        (
            "realization",
            vec!["realization"],
            vec![
                text("target"),
                T::enumeration("ModelingRealizationPolicy").with_name("policy"),
                text("accelerator").optional(),
            ],
        ),
    ];
    enumeration(
        builder,
        "ModelingDeclarationKind",
        arms.iter().flat_map(|(_, tags, _)| tags.iter().copied()),
    );
    enumeration(
        builder,
        "ModelingAccumulatorMode",
        ["conservation", "accounting"],
    );
    enumeration(
        builder,
        "ExternalDerivativeSource",
        ["analytic", "symbolic", "automatic", "supplied", "implicit"],
    );
    enumeration(
        builder,
        "ModelingContributionRole",
        [
            "inflow",
            "outflow",
            "generation",
            "consumption",
            "accumulation",
            "transfer",
            "positive",
            "negative",
        ],
    );
    enumeration(
        builder,
        "ModelingAnalysisRoute",
        ["steady", "integrated", "simultaneous"],
    );
    enumeration(
        builder,
        "ModelingFixtureBinding",
        ["value", "fix", "free", "lower", "upper"],
    );
    enumeration(
        builder,
        "ModelingFixtureExecution",
        [
            "pure",
            "steady",
            "initialized",
            "integrated",
            "simultaneous",
        ],
    );
    enumeration(
        builder,
        "ModelingRealizationPolicy",
        ["inline", "nested", "accelerated"],
    );
    let alternative = TaggedAlternative::new(
        "kind",
        arms.iter()
            .flat_map(|(arm, tags, _)| tags.iter().map(|tag| ((*tag).into(), (*arm).into()))),
    );
    let mut payload = vec![T::enumeration("ModelingDeclarationKind").with_name("kind")];
    payload.extend(
        arms.into_iter()
            .map(|(name, _, fields)| T::structure(fields).with_name(name).optional()),
    );
    relation(
        builder,
        N::Authored,
        "modeling_declarations",
        S::Model,
        &["declaration_id"],
        vec![
            column("declaration_id", T::id()),
            column("document_id", T::id()),
            column("parent_id", T::id()).optional(),
            column("ordinal", T::nonnegative(i64::MAX)),
            column("name", T::native(D::Utf8)),
            column("is_override", flag("is_override")),
            column("source_start", T::nonnegative(i64::from(u32::MAX))),
            column("source_end", T::nonnegative(i64::from(u32::MAX))),
            column(
                "value",
                T::structure(payload).with_alternative(&alternative),
            ),
        ],
        "Version-one generic modeling declaration. Exactly one tagged payload is present; parent references preserve lexical ownership. Expressions use the shared DSL, not another numerical IR.",
    );
    enumeration(
        builder,
        "ModelingCheckKind",
        [
            "expectation",
            "check",
            "original_equation",
            "closure",
            "validity",
        ],
    );
    relation(
        builder,
        N::Runtime,
        "modeling_checks",
        S::Derived,
        &["run_id", "step", "sample_index", "target_id", "source_id", "kind"],
        vec![
            column("run_id", T::id()),
            column("step", T::nonnegative(i64::MAX)),
            column("sample_index", T::nonnegative(i64::MAX)),
            column("time", T::native(D::Float64)).optional(),
            column("target_id", T::id()),
            column("source_id", T::id()),
            column("kind", T::enumeration("ModelingCheckKind")),
            column("value", T::native(D::Float64)),
            column("tolerance", T::native(D::Float64)).optional(),
            column("satisfied", T::native(D::Boolean)),
            column("within_validity", T::native(D::Boolean)).optional(),
            column("extrapolation_allowed", T::native(D::Boolean)).optional(),
        ],
        "Independent model checks supplement native outcomes. Step identifies the requested solve within a finite sequence; standalone analyses use zero. Static checks use sample_index zero without time; trajectory checks identify the requested sample and physical time in seconds. Validity membership and permission to extrapolate remain distinct observations.",
    );
    relation(
        builder,
        N::Runtime,
        "modeling_reports",
        S::Derived,
        &["run_id", "step", "target_id", "source_id"],
        vec![
            column("run_id", T::id()),
            column("step", T::nonnegative(i64::MAX)),
            column("target_id", T::id()),
            column("source_id", T::id()),
            column("label", T::native(D::Utf8)),
            column("path", T::native(D::Utf8)),
            column("quantity_id", T::id()),
            column("unit_id", T::id()),
            column("value", T::native(D::Float64)),
        ],
        "Canonical physical observations keyed by source and semantic target. Indexed members may share a presentation label without losing their coordinates.",
    );
    enumeration(
        builder,
        "ModelingConformanceKind",
        [
            "coverage",
            "preparation",
            "degrees_of_freedom",
            "derivatives",
            "envelope",
            "start_to_solve",
            "closure",
            "expectation",
            "check",
        ],
    );
    enumeration(
        builder,
        "ModelingConformanceStatus",
        [
            "passed",
            "failed",
            "inconclusive",
            "not_applicable",
            "cancelled",
            "unattempted",
        ],
    );
    relation(
        builder,
        N::Runtime,
        "modeling_conformance",
        S::Derived,
        &[
            "run_id",
            "fixture_id",
            "sample_index",
            "target_id",
            "source_id",
            "kind",
        ],
        vec![
            column("run_id", T::id()),
            column("fixture_id", T::id()),
            column("sample_index", T::nonnegative(i64::MAX)),
            column("time", T::native(D::Float64)).optional(),
            column("target_id", T::id()),
            column("source_id", T::id()),
            column("kind", T::enumeration("ModelingConformanceKind")),
            column("status", T::enumeration("ModelingConformanceStatus")),
            column("message", T::native(D::Utf8)),
            column("failure_ordinal", T::nonnegative(i64::MAX)).optional(),
            column("oracle_reference", T::native(D::Utf8)).optional(),
            column("oracle_revision", T::native(D::Utf8)).optional(),
        ],
        "Bounded shared checks over authored fixtures. Oracle links identify asserted source values; they do not claim an upstream run. Uncovered concrete definitions and incomplete samples are explicit.",
    );
    relation(
        builder,
        N::Runtime,
        "modeling_fixture_status",
        S::Derived,
        &["run_id", "fixture_id"],
        vec![
            column("run_id", T::id()),
            column("fixture_id", T::id()),
            column("status", T::enumeration("ModelingConformanceStatus")),
        ],
        "Every discovered fixture retains an aggregate disposition even when the detailed check limit or memory budget prevents further checks. Unattempted identities never disappear from an incomplete report.",
    );
}
