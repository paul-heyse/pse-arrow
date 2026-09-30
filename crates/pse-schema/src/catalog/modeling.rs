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
/// ADR-0123 Outcome 1: a type expression is a post-order arena. Every node's children
/// precede it and the last node is the root; names occur only as path segments.
fn type_arena(name: &str) -> T {
    T::list(
        T::structure(vec![
            T::enumeration("ModelingTypeNode").with_name("kind"),
            strings("path").optional(),
            text("name").optional(),
            T::structure(vec![
                T::native(D::Int16).with_name("num"),
                T::native(D::Int16).with_name("den"),
            ])
            .with_name("exponent")
            .optional(),
            T::list(T::native(D::UInt32)).with_name("children"),
        ])
        .named("ModelingTypeArenaNode"),
    )
    .with_name(name)
}
/// ADR-0123 Outcome 7: a typed requirement on a package version, shared by imports and
/// manifest dependencies.
pub(super) fn version_requirement() -> T {
    T::structure(vec![
        T::enumeration("ModelingVersionOperator").with_name("operator"),
        T::nonnegative(i64::MAX).with_name("major"),
        T::nonnegative(i64::MAX).with_name("minor"),
        T::nonnegative(i64::MAX).with_name("patch"),
    ])
    .named("VersionRequirement")
}
/// The scalar arms of a cell, each a named structure a key cell shares (ADR-0123
/// Outcome 1): a Boolean, an integer, a quantity as a magnitude and a canonical unit
/// product of `ModelingUnitFactor`s, text, an identifier of a scheme, and a reference by
/// path.
fn scalar_arms() -> Vec<T> {
    vec![
        T::structure(vec![flag("value")])
            .named("ModelingCellValueBoolean")
            .with_name("boolean")
            .optional(),
        T::structure(vec![T::native(D::Int64).with_name("value")])
            .named("ModelingCellValueInteger")
            .with_name("integer")
            .optional(),
        T::structure(vec![
            T::native(D::Float64).with_name("magnitude"),
            T::list(
                T::structure(vec![
                    text("symbol"),
                    T::native(D::Int16).with_name("num"),
                    T::native(D::Int16).with_name("den"),
                ])
                .named("ModelingUnitFactor"),
            )
            .with_name("unit")
            .optional(),
        ])
        .named("ModelingCellValueQuantity")
        .with_name("quantity")
        .optional(),
        T::structure(vec![text("value")])
            .named("ModelingCellValueText")
            .with_name("text")
            .optional(),
        T::structure(vec![strings("scheme"), text("value")])
            .named("ModelingCellValueIdentifier")
            .with_name("identifier")
            .optional(),
        T::structure(vec![strings("path")])
            .named("ModelingCellValueReference")
            .with_name("reference")
            .optional(),
    ]
}
const SCALAR_ARMS: [&str; 6] = [
    "boolean",
    "integer",
    "quantity",
    "text",
    "identifier",
    "reference",
];
/// ADR-0123 Outcome 1: a data cell is a typed tagged value, parsed once and typed at
/// admission; it is never an expression. A quantity is a magnitude and a canonical unit
/// product; a bare number has no unit and is the neutral scalar, while a written unit,
/// `{1}` included, resolves against the expected quantity type. Names occur only as path
/// segments. A keyed-row reference names a keyed kind or a table and the row's key cells,
/// each a scalar key cell, and is resolved at admission (Plan 23 KR5). Any cell may carry
/// an uncertainty, which admission accepts only on a numeric value.
fn cell(name: &str) -> T {
    let mut fields = vec![T::enumeration("ModelingCellKind").with_name("kind")];
    fields.extend(scalar_arms());
    fields.push(
        T::structure(vec![T::list(T::structure(vec![strings("path")])).with_name("paths")])
            .with_name("references")
            .optional(),
    );
    fields.push(
        T::structure(vec![
            strings("target"),
            T::list(key_cell()).with_name("keys"),
        ])
        .with_name("row")
        .optional(),
    );
    T::structure(vec![
        T::structure(fields)
            .with_alternative(
                &TaggedAlternative::new(
                    "kind",
                    SCALAR_ARMS
                        .into_iter()
                        .chain(["references", "row"])
                        .map(|arm| (arm.into(), arm.into())),
                )
                .with_unit("missing"),
            )
            .with_name("value"),
        T::structure(vec![
            T::enumeration("ModelingUncertaintyKind").with_name("kind"),
            T::native(D::Float64).with_name("magnitude"),
        ])
        .with_name("uncertainty")
        .optional(),
    ])
    .named("ModelingCell")
    .with_name(name)
}
/// A key cell of a keyed-row reference: one scalar arm, never absent, never a set or a
/// further row reference, and without an uncertainty (Plan 23 KR5).
fn key_cell() -> T {
    let mut fields = vec![T::enumeration("ModelingKeyCellKind").with_name("kind")];
    fields.extend(scalar_arms());
    T::structure(fields)
        .with_alternative(&TaggedAlternative::new(
            "kind",
            SCALAR_ARMS.map(|arm| (arm.into(), arm.into())),
        ))
        .named("ModelingKeyCell")
}
/// An inclusive integer range: the declared values of an integer-range key, or the
/// completeness set of an integer key (Plan 23 KR5).
fn integer_range(name: &str) -> T {
    T::structure(vec![
        T::native(D::Int64).with_name("lower"),
        T::native(D::Int64).with_name("upper"),
    ])
    .named("ModelingIntegerRange")
    .with_name(name)
}
/// ADR-0123 Outcome 3: a relation's completeness, one entry per key. A key over a set
/// names a declared set or an enumeration; a key over an integer range names its bounds;
/// a key with neither is open on the relation, and each dataset that claims completeness
/// names the set it covers.
fn completeness(name: &str) -> T {
    T::list(
        T::structure(vec![
            text("key"),
            strings("set").optional(),
            integer_range("range").optional(),
        ])
        .named("ModelingCompleteness"),
    )
    .with_name(name)
}
/// ADR-0123 Outcome 5: the provenance of a dataset or a constant, each part by path. The
/// source names an entity whose kind, or an ancestor kind, carries the provenance facet;
/// the role names a member of a package enumeration, whose declared data facets the kernel
/// acts on; the lineage names the datasets and sources the data derive from, acyclic.
fn provenance(name: &str) -> T {
    T::structure(vec![
        strings("source"),
        strings("role"),
        T::list(
            T::structure(vec![
                T::enumeration("ModelingLineageKind").with_name("kind"),
                strings("path"),
            ])
            .named("ModelingLineageEntry"),
        )
        .with_name("lineage"),
    ])
    .named("ModelingProvenance")
    .with_name(name)
}
/// ADR-0123 Outcome 4: a relation's validity envelopes, declared as data. Each names its
/// axis, the axis quantity type and the two typed columns bounding it; data declares no
/// extrapolation policy.
fn envelopes(name: &str) -> T {
    T::list(
        T::structure(vec![
            text("name"),
            type_arena("type"),
            text("lower"),
            text("upper"),
        ])
        .named("ModelingEnvelope"),
    )
    .with_name(name)
}
/// ADR-0123 Outcome 4: the envelopes a function guards. Each names the argument carrying a
/// row or entity, the envelope that row's relation or kind declares, and the arguments it
/// guards: one point, or the two endpoints of an integration interval.
fn guards(name: &str) -> T {
    T::list(
        T::structure(vec![
            text("carrier"),
            text("envelope"),
            T::enumeration("ModelingEnvelopeExtent").with_name("extent"),
            strings("arguments"),
        ])
        .named("ModelingEnvelopeGuard"),
    )
    .with_name(name)
}
fn parameters(name: &str) -> T {
    T::list(T::structure(vec![
        text("name"),
        type_arena("type"),
        text("default_value").optional(),
    ]))
    .with_name(name)
}
fn indices() -> T {
    T::list(T::structure(vec![text("name"), text("domain")])).with_name("indices")
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
                // ADR-0123 Outcome 5: a test names the source entity its expected values
                // come from, by path; the entity's kind carries the provenance facet.
                strings("oracle").optional(),
                // ADR-0123 Outcome 5: an entity kind's facets; `provenance` marks the kinds
                // whose entities may be sources, refinements included.
                T::list(T::enumeration("ModelingKindFacet")).with_name("facets"),
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
                    // ADR-0119: the fixture's execution policy over the run's. Each absent
                    // setting keeps the run's: an explicit backend, presolve auto or off,
                    // the derivative inspection's step, tolerance and cell allowance, the
                    // specialization allowances and the foreign-library allowance of the
                    // fixture's solves. Scientific data is never here.
                    T::structure(vec![
                        T::enumeration("NativeBackend")
                            .with_name("backend")
                            .optional(),
                        T::enumeration("PresolvePolicyKind")
                            .with_name("presolve")
                            .optional(),
                        T::native(D::Float64)
                            .with_name("derivative_step")
                            .optional(),
                        T::native(D::Float64)
                            .with_name("derivative_tolerance")
                            .optional(),
                        T::nonnegative(i64::MAX)
                            .with_name("derivative_cells")
                            .optional(),
                        T::nonnegative(i64::MAX).with_name("items").optional(),
                        T::nonnegative(i64::MAX)
                            .with_name("body_occurrences")
                            .optional(),
                        T::nonnegative(i64::MAX)
                            .with_name("body_slots")
                            .optional(),
                        T::nonnegative(i64::MAX)
                            .with_name("foreign_bytes")
                            .optional(),
                    ])
                    .with_name("policy")
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
                        // on the integrated axis and one value per interval. A schedule held
                        // free is a shooting control (ADR-0110 Outcome 5): its values are the
                        // starting guesses, within its optional bounds.
                        T::list(T::structure(vec![
                            text("target"),
                            strings("times"),
                            strings("values"),
                            flag("free"),
                            text("lower").optional(),
                            text("upper").optional(),
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
                        // ADR-0123 Outcome 1: a fact is named in its registry namespace.
                        T::list(T::structure(vec![
                            T::enumeration("ModelingFactNamespace").with_name("namespace"),
                            text("name"),
                            flag("value"),
                        ]))
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
                    // ADR-0110 Outcome 5: a shooting fixture's method and, for multiple
                    // shooting, its inner nodes on the integrated axis.
                    T::structure(vec![
                        T::enumeration("ShootingMethod").with_name("method"),
                        strings("nodes"),
                    ])
                    .with_name("shooting")
                    .optional(),
                    // Plan 23 H5: an expected failure is its typed class and its lineage.
                    // A rejected validity predicate names its layer; a form or data layer
                    // predicate its form, the parameter sets bounding it and the form's
                    // arguments it constrains; a closure range the members it bounds. A
                    // structural refusal or a diagnostic finding names its members.
                    // Exactly one lineage is present.
                    T::structure(vec![
                        T::enumeration("NativeBoundaryClass").with_name("class"),
                        T::structure(vec![
                            T::enumeration("ModelingValidityLayer").with_name("layer"),
                            text("form").optional(),
                            strings("sets"),
                            strings("variables"),
                        ])
                        .with_name("validity")
                        .optional(),
                        strings("members"),
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
                "set",
                "child",
                "port",
                "preset",
                "scope_value",
            ],
            vec![
                type_arena("type").optional(),
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
                type_arena("return_type"),
                text("body").optional(),
                // The form layer: the function's own domain, which never extrapolates.
                text("validity").optional(),
                // The data layer: the envelopes of row or entity arguments this function
                // guards (ADR-0123 Outcome 4).
                guards("guards"),
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
        // ADR-0123 Outcome 3: a relation with constraints. Keys form row identity; an
        // integer key may declare its range. A derived column carries the expression
        // evaluated once per row with the keys and columns bound. `required` exactly
        // declares completeness; `default` exactly carries its typed cell. A symmetric key
        // pair stores each unordered pair once; uniqueness names keys and supplied columns;
        // each requirement is a predicate every row satisfies. An envelope bounds an axis by
        // two typed columns (Outcome 4).
        (
            "table",
            vec!["table"],
            vec![
                T::list(T::structure(vec![
                    text("name"),
                    type_arena("type"),
                    integer_range("range").optional(),
                ]))
                .with_name("keys"),
                T::list(T::structure(vec![
                    text("name"),
                    type_arena("type"),
                    text("derived").optional(),
                ]))
                .with_name("columns"),
                // Present exactly when the table declares no columns.
                type_arena("value_type").optional(),
                T::enumeration("ModelingMissingPolicy").with_name("missing_policy"),
                cell("default_value").optional(),
                completeness("complete_over"),
                T::structure(vec![
                    text("first"),
                    text("second"),
                    T::enumeration("ModelingDiagonalPolicy").with_name("diagonal"),
                ])
                .with_name("symmetry")
                .optional(),
                T::list(T::structure(vec![strings("names")])).with_name("unique"),
                strings("requirements"),
                envelopes("envelopes"),
            ],
        ),
        // ADR-0123 Outcome 4: an entity kind's validity envelope, declared as data: the
        // declaration names the axis; its quantity type and the two attributes bounding it,
        // declared by the kind or inherited, follow.
        (
            "envelope",
            vec!["envelope"],
            vec![type_arena("type"), text("lower"), text("upper")],
        ),
        // ADR-0123 Outcome 4: the extrapolation policy a property package (a definition) or
        // an analysis (a test or a case) selects for one validity layer of its instances.
        (
            "extrapolation",
            vec!["extrapolation"],
            vec![
                T::enumeration("ModelingValidityLayer").with_name("layer"),
                T::enumeration("ExtrapolationPolicy").with_name("policy"),
            ],
        ),
        // ADR-0123 Outcome 2: an entity kind's attribute. A declaration carries its type, a
        // key flag and an optional default; a binding of an inherited attribute carries no
        // type and binds its value for the kind and its refinements. A unique attribute holds
        // distinct values across every entity of its kind; a derived attribute carries the
        // expression admission evaluates once per entity, with the entity and its attributes
        // bound, and no default (Plan 23 D0).
        (
            "attribute",
            vec!["attribute"],
            vec![
                type_arena("type").optional(),
                flag("key"),
                cell("value").optional(),
                flag("unique"),
                text("derived").optional(),
            ],
        ),
        // Rows of a table or of a keyed entity kind: positional cells. A keyed kind's key
        // the dataset supplies for every row (for example its source) is a declared binding.
        // A dataset of a table with open completeness keys may claim that its rows cover a
        // declared set for each open key (ADR-0123 Outcome 3). Every dataset names its
        // provenance (Outcome 5).
        (
            "dataset",
            vec!["dataset"],
            vec![
                text("target"),
                provenance("provenance"),
                T::list(T::structure(vec![text("name"), cell("value")])).with_name("bindings"),
                completeness("complete_over"),
                T::list(T::structure(vec![
                    T::list(cell("key")).with_name("keys"),
                    T::list(cell("value")).with_name("values"),
                ]))
                .with_name("rows"),
            ],
        ),
        (
            "entity",
            vec!["entity"],
            vec![
                text("kind_name"),
                T::list(T::structure(vec![text("name"), cell("value")])).with_name("attributes"),
            ],
        ),
        // Members have identities, so renaming a member re-keys nothing (ADR-0123 Outcome 2).
        // A member may declare the data facets the kernel acts on when a dataset or a
        // constant names it as its role (Outcome 5).
        (
            "enumeration",
            vec!["enum"],
            vec![
                T::list(T::structure(vec![
                    T::id().with_name("member_id"),
                    text("name"),
                    T::list(T::enumeration("ModelingDataFacet")).with_name("facets"),
                ]))
                .with_name("members"),
            ],
        ),
        (
            "constant",
            vec!["constant"],
            vec![type_arena("type"), cell("value"), provenance("provenance")],
        ),
        (
            "import",
            vec!["import"],
            vec![
                // ADR-0123 Outcome 7: the typed requirement on the imported package's version.
                version_requirement().with_name("version"),
                text("alias").optional(),
            ],
        ),
        ("guard", vec!["when"], vec![text("predicate")]),
        (
            "accumulator",
            vec!["accumulator"],
            vec![
                indices(),
                type_arena("type"),
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
                T::enumeration("ModelingAnnotationKind").with_name("kind"),
                text("target"),
                strings("arguments"),
                // ADR-0115 Outcome 3: present exactly on `valid`, whose arguments are then
                // its lower and upper endpoints.
                T::enumeration("ExtrapolationPolicy")
                    .with_name("extrapolation")
                    .optional(),
                // Present exactly on `scale`, whose arguments are then empty.
                T::enumeration("ConstraintScalingScheme")
                    .with_name("scheme")
                    .optional(),
                // Present exactly on `connectivity`, whose arguments are then empty; an
                // absent maximum admits any number of connections.
                T::structure(vec![
                    T::nonnegative(i64::from(u32::MAX))
                        .with_name("incoming")
                        .optional(),
                    T::nonnegative(i64::from(u32::MAX))
                        .with_name("outgoing")
                        .optional(),
                ])
                .with_name("connectivity")
                .optional(),
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
            vec![type_arena("type"), text("lower"), text("upper")],
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
    // ADR-0123 Outcome 2: an identifier scheme is declared by name alone; its values are
    // opaque and unique within the package closure.
    let units = ["identifier_scheme"];
    enumeration(
        builder,
        "ModelingDeclarationKind",
        arms.iter()
            .flat_map(|(_, tags, _)| tags.iter().copied())
            .chain(units),
    );
    enumeration(
        builder,
        "ModelingCellKind",
        [
            "boolean",
            "integer",
            "quantity",
            "text",
            "identifier",
            "reference",
            "references",
            "row",
            "missing",
        ],
    );
    // The scalar arms a key cell of a keyed-row reference may take (Plan 23 KR5).
    enumeration(builder, "ModelingKeyCellKind", SCALAR_ARMS);
    // A standard uncertainty and a bound are in the value's unit; a relative uncertainty is
    // a dimensionless fraction of the value.
    enumeration(
        builder,
        "ModelingUncertaintyKind",
        ["standard", "relative", "bound"],
    );
    // ADR-0123 Outcome 1: the node kinds of a type arena.
    enumeration(
        builder,
        "ModelingTypeNode",
        [
            "boolean",
            "integer",
            "text",
            "named",
            "variable",
            "optional",
            "set",
            "row",
            "table",
            "tuple",
            "indexed",
            "function",
            "argument",
            "delta",
            "product",
            "quotient",
            "power",
            "identifier",
            // Generic physical references: a named quantity type or reference state of
            // the physical document (ADR-0123 Outcome 6).
            "quantity_type",
            "reference_state",
        ],
    );
    // ADR-0123 Outcome 1: the vocabulary the kernel acts on is registry enums.
    enumeration(
        builder,
        "ModelingMissingPolicy",
        ["required", "optional", "default"],
    );
    // ADR-0123 Outcome 1: key symmetry. A symmetric key pair either admits a row whose two
    // keys are equal or excludes the diagonal (Plan 23 KR5).
    enumeration(builder, "ModelingDiagonalPolicy", ["allowed", "excluded"]);
    enumeration(
        builder,
        "ModelingAnnotationKind",
        [
            "start",
            "nominal",
            "bounds",
            "scale",
            "report",
            "valid",
            "check",
            "objective",
            "connectivity",
        ],
    );
    // The reserved fact namespaces: the analysis route and its derived dynamic flag, the
    // selected objective level and the selected initialization stages.
    enumeration(
        builder,
        "ModelingFactNamespace",
        ["analysis", "objective", "stage"],
    );
    // Phase 0 and 1 admit exact version requirements only (§6.1).
    enumeration(builder, "ModelingVersionOperator", ["exact"]);
    // ADR-0123 Outcome 5: the data facets a role declares. Test-only data may be read only
    // by test fixtures; data requiring lineage names what it derives from. Roles themselves
    // are package enumeration members.
    enumeration(
        builder,
        "ModelingDataFacet",
        ["test_only", "requires_lineage"],
    );
    // ADR-0123 Outcome 5: the facets of an entity kind. An entity is a source exactly when
    // its kind or an ancestor kind carries `provenance`.
    enumeration(builder, "ModelingKindFacet", ["provenance"]);
    // ADR-0123 Outcome 5: what a lineage entry names: a dataset or a source entity.
    enumeration(builder, "ModelingLineageKind", ["dataset", "source"]);
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
    // A validity layer either refuses a value outside it or, explicitly selected by its
    // consumer, accepts it as an extrapolation (ADR-0115 Outcome 3, ADR-0123 Outcome 4).
    enumeration(builder, "ExtrapolationPolicy", ["reject", "extrapolate"]);
    // ADR-0123 Outcome 4: validity is the intersection of three layers. The form layer is a
    // function's own domain and never extrapolates; the data layer is the envelopes a
    // relation or kind declares; the closure layer is an `annotation valid` range.
    enumeration(
        builder,
        "ModelingValidityLayer",
        ["form", "data", "closure"],
    );
    // ADR-0123 Outcome 4: what an envelope guard covers: one argument, or the closed
    // interval between two, such as the integration interval [T0, T] of an increment.
    enumeration(builder, "ModelingEnvelopeExtent", ["point", "interval"]);
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
    let alternative = units.into_iter().fold(
        TaggedAlternative::new(
            "kind",
            arms.iter()
                .flat_map(|(arm, tags, _)| tags.iter().map(|tag| ((*tag).into(), (*arm).into()))),
        ),
        TaggedAlternative::with_unit,
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
        15,
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
        "Generic modeling declaration. Exactly one tagged payload is present; parent references preserve lexical ownership. Expressions use the shared DSL, not another numerical IR. Version two adds the declared domain of a variable binding (ADR-0103); every other binding carries none. Version three adds indicator conditions, ordered sets, cardinality, piecewise-linear, logic and disjunction declarations and their realization arguments (ADR-0104). Version four adds a fixture's declared solve intent (ADR-0119). Version five adds the typed members of an objective annotation: sense, priority, weight, normalization and its level's absolute and relative degradation tolerances (ADR-0111); complementarity declarations; and a realization's smoothing function (ADR-0104). Version six adds an integration fixture's scheduled inputs: each schedule's target, change times and one value per interval; a fixture's same-layout modes, each with the facts that select it and its events: guard, crossing direction, tolerance, resets and successor mode (ADR-0119); and a shooting fixture's controls, schedules held free within optional bounds, with its shooting method and inner nodes (ADR-0110). Version seven adds a fixture's execution policy: an explicit backend, presolve auto or off, derivative inspection step, tolerance and cells, and specialization item, body-occurrence and body-slot allowances, each replacing the run's for that fixture only (ADR-0119). Version eight adds the policy's foreign-library allowance in bytes, which the fixture's solves reserve and a native library that enforces its own memory limit receives, in place of the deployment's (ADR-0119). Version nine is structured (ADR-0123 Outcome 1): every type is a post-order type arena whose children precede their parent and whose last node is the root; table absence, annotation kinds and fact namespaces are registry enums, a validity annotation carries its typed extrapolation policy, a scaling annotation its scheme and a connectivity annotation its typed maxima; an import carries its typed version requirement (Outcome 7). Version ten makes entities typed records (ADR-0123 Outcome 2): data cells are typed tagged values (boolean, integer, quantity as magnitude and unit product, text, identifier, reference, references or missing, each with an optional uncertainty), parsed once and never expressions; entity attribute values, attribute defaults, kind-level bindings of inherited attributes, dataset rows and typed constants are cells; an attribute declares whether it is a key; a dataset names its target table or keyed kind and binds the keys it supplies for every row; enumeration members carry identities; and identifier schemes are declared. Version eleven gives relations constraints (ADR-0123 Outcome 3): a table key may declare an inclusive integer range; a column may be derived by an expression evaluated once per row; the default of the default policy is a typed cell; a required table declares its completeness, one entry per key over a declared set, an enumeration or an integer range, or open for datasets to claim; a symmetric key pair declares its diagonal policy; uniqueness constraints name keys and supplied columns; row requirements are predicates; a dataset may claim completeness over declared sets for its table's open keys; and a cell may reference a keyed row or a table row by its target and key cells. Version twelve types provenance (ADR-0123 Outcome 5): every dataset and constant names its source entity, its role and its lineage by path, in place of a source text; an entity kind declares its facets, and an entity is a source exactly when its kind or an ancestor kind carries the provenance facet; an enumeration member declares the data facets the kernel acts on when it is named as a role, test-only or requiring lineage; a lineage entry names a dataset or a source; and a test names the source entity of its expected values as its oracle, in place of a reference and revision text. Version thirteen types validity envelopes (ADR-0123 Outcome 4): a table declares its envelopes, each an axis with its quantity type bounded by two typed columns, and an entity kind declares an envelope declaration bounded by two of its attributes, in place of bounds read from columns named minimum and maximum; data declares no extrapolation policy; a function declares which of its arguments, or which integration interval, each envelope of a row or entity argument guards; and a definition, test or case selects the extrapolation policy of the data layer for its instances. Version fourteen completes entity kinds (Plan 23 D0): an attribute may be unique, its values distinct across every entity of its kind and its refinements, and may be derived by an expression admission evaluates once per entity with the entity and its attributes bound, in place of a supplied value; a requirement declared in an entity kind is a predicate every entity of the kind satisfies. Version fifteen types a fixture's expected failure (Plan 23 H5): its boundary class and its lineage, in place of a rule text. A rejected validity predicate names its layer: a form or data layer predicate names its form by path, the parameter sets bounding it as static expressions and the form's arguments it constrains by name; a closure range names no form and no set, and its variables are the member paths it bounds. A structural refusal or a diagnostic finding names the members it concerns by path. Exactly one lineage is present.",
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
        3,
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
            column("layer", T::enumeration("ModelingValidityLayer")).optional(),
        ],
        "Independent model checks supplement native outcomes. Step identifies the requested solve within a finite sequence; standalone analyses use zero. Static checks use sample_index zero without time; trajectory checks identify the requested sample and physical time in seconds. Validity membership and permission to extrapolate remain distinct observations. Version two adds basis: point for a check evaluated at the step's point, global_bound for an objective-bound check evaluated against the step's certified dual bound (ADR-0119); a point result states no global property. Version three adds the validity layer a validity check observes, present exactly on validity checks (ADR-0123 Outcome 4): closure for an annotated range, whose source is the annotation, and data for a declared envelope whose consumer selected extrapolation, whose source is the relation or kind declaring it. The form layer never extrapolates, so a value outside it is a rejected evaluation, not a check.",
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
    relation_version(
        builder,
        N::Runtime,
        "modeling_conformance",
        2,
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
            column("oracle_source_id", T::id())
                .with_identity("declaration")
                .optional(),
        ],
        "Bounded shared checks over authored fixtures. The oracle source is the entity a fixture names as the source of its expected values; it identifies asserted source values and does not claim an upstream run. Uncovered concrete definitions and incomplete samples are explicit. Version two replaces the verbatim oracle reference and revision text with the oracle's source entity identity (ADR-0123 Outcome 5).",
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
            ("runtime.modeling_conformance", "oracle_source_id", "declaration"),
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
