// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generic modeling-language declarations; the sole durable kernel IR schema.
use super::declarations::{column, enumeration, identity, relation, relation_version, run_id};
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
                "disjunction",
                "alternative",
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
                    // ADR-0119 Outcome 1: the fixture's declared solve intent; absent leaves
                    // the intent to the runtime policy.
                    T::enumeration("NativeSolveIntent")
                        .with_name("intent")
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
                        // ADR-0119 Outcome 2: a piecewise-constant input, its change times
                        // on the integrated axis and one value per interval.
                        T::list(T::structure(vec![
                            text("target"),
                            strings("times"),
                            strings("values"),
                        ]))
                        .with_name("schedules"),
                    ])
                    .with_name("integration")
                    .optional(),
                    // ADR-0119 Outcome 3: same-layout modes in order (the first starts the
                    // integration), the Boolean facts selecting each and its events. An event
                    // without a successor mode is terminal.
                    T::list(T::structure(vec![
                        text("name"),
                        T::list(T::structure(vec![text("name"), flag("value")]))
                            .with_name("facts"),
                        T::list(T::structure(vec![
                            text("guard"),
                            T::enumeration("EventDirection").with_name("direction"),
                            text("tolerance"),
                            T::list(T::structure(vec![text("target"), text("expression")]))
                                .with_name("reset"),
                            text("next").optional(),
                        ]))
                        .with_name("events"),
                    ]))
                    .with_name("modes"),
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
                // ADR-0103: present exactly on a variable; continuous is the default spelling.
                T::enumeration("ModelingVariableDomain")
                    .with_name("domain")
                    .optional(),
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
            vec![
                indices(),
                text("expression"),
                // ADR-0104: an indicator constraint holds only when the binary takes `active`.
                T::structure(vec![text("variable"), flag("active")])
                    .with_name("condition")
                    .optional(),
            ],
        ),
        (
            "ordered_set",
            vec!["sos1", "sos2"],
            vec![indices(), text("member"), text("weight")],
        ),
        (
            "cardinality",
            vec!["atmost", "atleast", "exactly"],
            vec![indices(), text("count"), text("member")],
        ),
        (
            "piecewise",
            vec!["piecewise"],
            vec![
                indices(),
                text("output"),
                text("input"),
                text("abscissa"),
                text("ordinate"),
            ],
        ),
        ("logic", vec!["logic"], vec![indices(), text("proposition")]),
        // ADR-0104 §5: `complements name[i in s]: (first >= 0, second >= 0);`, the pair
        // 0 <= first ⊥ second >= 0, realized by smooth, penalty_l1 or disjunctive.
        (
            "complementarity",
            vec!["complementarity"],
            vec![indices(), text("first"), text("second")],
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
                // ADR-0111: the typed members of `annotation objective`, present exactly on
                // an objective, whose arguments are then empty. Priority orders levels
                // (lower first); weight and normalization compose a level's weighted sum;
                // the tolerances bound the level's degradation for later levels.
                T::structure(vec![
                    T::enumeration("NativeObjectiveSense").with_name("sense"),
                    T::native(D::Int64).with_name("priority").optional(),
                    text("weight").optional(),
                    text("normalization").optional(),
                    text("absolute_tolerance").optional(),
                    text("relative_tolerance").optional(),
                ])
                .with_name("objective")
                .optional(),
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
                // An authored big-M, a derived big-M's relative margin, a hull's epsilon or
                // a smoothed complementarity's width.
                text("argument").optional(),
                // The authored smoothing function f(first, second, width) a smooth
                // complementarity equates to zero; present exactly on `smooth`.
                text("function").optional(),
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
        "ModelingVariableDomain",
        [
            "continuous",
            "integer",
            "binary",
            "semicontinuous",
            "semiinteger",
        ],
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
            "shooting",
        ],
    );
    // An `annotation valid` range either refuses a value outside it or, explicitly
    // selected, accepts it as an extrapolation (ADR-0115 Outcome 3).
    enumeration(builder, "ExtrapolationPolicy", ["reject", "extrapolate"]);
    // ADR-0104: constraint forms and disjunctions name their realization.
    enumeration(
        builder,
        "ModelingRealizationPolicy",
        [
            "inline",
            "nested",
            "accelerated",
            "big_m",
            "derived_big_m",
            "hull",
            "indicator",
            "linear",
            "native",
            "sos2",
            "incremental",
            "smooth",
            "penalty_l1",
            "disjunctive",
        ],
    );
    // ADR-0104 §5: a requirement a lowering places on the solve route, carried by the case
    // structure. An authored `penalty(l1)` requires the l1 exact-penalty route.
    enumeration(
        builder,
        "ModelingStructuralRequirement",
        ["l1_exact_penalty"],
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
    identity(
        builder,
        "declaration",
        "One authored modeling declaration. A specialization root, a definition and a member are declarations in a role, not separate entities",
    );
    relation_version(
        builder,
        N::Authored,
        "modeling_declarations",
        6,
        S::Model,
        &["declaration_id"],
        vec![
            column("declaration_id", T::id()).with_owned_identity("declaration"),
            column("document_id", T::id()),
            column("parent_id", T::id())
                .with_identity("declaration")
                .optional(),
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
        "Generic modeling declaration. Exactly one tagged payload is present; parent references preserve lexical ownership. Expressions use the shared DSL, not another numerical IR. Version two adds the declared domain of a variable binding (ADR-0103); every other binding carries none. Version three adds indicator conditions, ordered sets, cardinality, piecewise-linear, logic and disjunction declarations and their realization arguments (ADR-0104). Version four adds a fixture's declared solve intent (ADR-0119). Version five adds the typed members of an objective annotation: sense, priority, weight, normalization and its level's absolute and relative degradation tolerances (ADR-0111); complementarity declarations; and a realization's smoothing function (ADR-0104). Version six adds an integration fixture's scheduled inputs: each schedule's target, change times and one value per interval; and a fixture's same-layout modes, each with the facts that select it and its events: guard, crossing direction, tolerance, resets and successor mode (ADR-0119).",
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
    // ADR-0119 Outcome 5: what a check was established on. An objective-bound check reads
    // the step's certified dual bound when the step carries one; every other check, and an
    // objective-bound check without one, is evaluated at the point.
    enumeration(builder, "ModelingCheckBasis", ["point", "global_bound"]);
    relation_version(
        builder,
        N::Runtime,
        "modeling_checks",
        2,
        S::Derived,
        &[
            "run_id",
            "step",
            "sample_index",
            "target_id",
            "source_id",
            "kind",
        ],
        vec![
            run_id(),
            column("step", T::nonnegative(i64::MAX)),
            column("sample_index", T::nonnegative(i64::MAX)),
            column("time", T::native(D::Float64)).optional(),
            column("target_id", T::id()),
            column("source_id", T::id()).with_identity("declaration"),
            column("kind", T::enumeration("ModelingCheckKind")),
            column("value", T::native(D::Float64)),
            column("tolerance", T::native(D::Float64)).optional(),
            column("satisfied", T::native(D::Boolean)),
            column("within_validity", T::native(D::Boolean)).optional(),
            column("extrapolation_allowed", T::native(D::Boolean)).optional(),
            column("basis", T::enumeration("ModelingCheckBasis")),
        ],
        "Independent model checks supplement native outcomes. Step identifies the requested solve within a finite sequence; standalone analyses use zero. Static checks use sample_index zero without time; trajectory checks identify the requested sample and physical time in seconds. Validity membership and permission to extrapolate remain distinct observations. Version two adds basis: point for a check evaluated at the step's point, global_bound for an objective-bound check evaluated against the step's certified dual bound (ADR-0119); a point result states no global property.",
    );
    relation(
        builder,
        N::Runtime,
        "modeling_reports",
        S::Derived,
        &["run_id", "step", "target_id", "source_id"],
        vec![
            run_id(),
            column("step", T::nonnegative(i64::MAX)),
            column("target_id", T::id()),
            column("source_id", T::id()).with_identity("declaration"),
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
            run_id(),
            column("fixture_id", T::id()).with_identity("declaration"),
            column("sample_index", T::nonnegative(i64::MAX)),
            column("time", T::native(D::Float64)).optional(),
            column("target_id", T::id()),
            column("source_id", T::id()).with_identity("declaration"),
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
            run_id(),
            column("fixture_id", T::id()).with_identity("declaration"),
            column("status", T::enumeration("ModelingConformanceStatus")),
        ],
        "Every discovered fixture retains an aggregate disposition even when the detailed check limit or memory budget prevents further checks. Unattempted identities never disappear from an incomplete report.",
    );
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "assertions over the platform registry")]

    use crate::model::FieldContract;

    /// Modeling identities (Plan 22 B7, B3c): packages and declarations are owned by
    /// their keys and inherited through references; instances are carried without an
    /// owning key. A model and a case are declarations: every lineage, solve and
    /// requirement row names them as `declaration`, and the instance or fit it solved in
    /// columns of their own.
    #[test]
    fn modeling_identities_declared() {
        let registry = crate::registry().unwrap();
        let owner = |name: &str| {
            let owner = registry.identity(name).unwrap().owner.as_ref()?;
            Some((owner.relation.clone(), owner.column.clone()))
        };
        assert_eq!(
            owner("package"),
            Some(("authored.packages".to_owned(), "package_id".to_owned()))
        );
        assert_eq!(
            owner("declaration"),
            Some((
                "authored.modeling_declarations".to_owned(),
                "declaration_id".to_owned()
            ))
        );
        assert_eq!(owner("instance"), None);
        for gone in ["model", "case"] {
            assert!(registry.identity(gone).is_none(), "{gone}");
        }
        let carried = |relation: &str, column: &str| {
            registry
                .relation(relation)
                .and_then(|spec| spec.column(column))
                .and_then(FieldContract::identity)
                .map(str::to_owned)
        };
        for (relation, column, identity) in [
            ("authored.documents", "package_id", "package"),
            ("normalized.package_graph", "package_id", "package"),
            ("reference.math_context", "package_id", "package"),
            ("authored.modeling_declarations", "parent_id", "declaration"),
            ("reference.reference_states", "subject_id", "declaration"),
            ("runtime.modeling_checks", "source_id", "declaration"),
            ("runtime.modeling_conformance", "fixture_id", "declaration"),
            ("authored.numerical_requirements", "model_id", "declaration"),
            ("authored.numerical_requirements", "case_id", "declaration"),
            ("authored.numerical_requirements", "instance_id", "instance"),
            ("authored.numerical_requirements", "fit_id", "fit"),
            ("runtime.run_lineage", "model_id", "declaration"),
            ("runtime.run_lineage", "case_id", "declaration"),
            ("runtime.run_lineage", "instance_id", "instance"),
            ("runtime.run_lineage", "fit_id", "fit"),
            ("runtime.solve_runs", "model_id", "declaration"),
            ("runtime.solve_runs", "case_id", "declaration"),
            ("runtime.solve_runs", "instance_id", "instance"),
            ("runtime.run_lineage", "run_id", "run"),
            ("runtime.solve_runs", "run_id", "run"),
            ("runtime.modeling_checks", "run_id", "run"),
            ("runtime.diagnostics_findings", "run_id", "run"),
        ] {
            assert_eq!(
                carried(relation, column).as_deref(),
                Some(identity),
                "{relation}.{column}"
            );
        }
        assert_eq!(carried("runtime.modeling_checks", "target_id"), None);
        let points = registry
            .relation("runtime.modeling_studies")
            .and_then(|spec| spec.column("points"))
            .unwrap()
            .children()
            .remove(0)
            .children();
        let nested = |name: &str| {
            points
                .iter()
                .find(|field| field.name() == name)
                .and_then(FieldContract::identity)
                .map(str::to_owned)
        };
        assert_eq!(nested("root_id").as_deref(), Some("declaration"));
        assert_eq!(nested("instance_id").as_deref(), Some("instance"));
    }
}
