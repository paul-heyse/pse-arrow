// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durable declarations and physical results for the native library compiler.
//! Expressions are authored DSL; no CAS serialization or evaluator state is durable.
use super::declarations::{column, enumeration, relation, relation_version};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;
fn text() -> T {
    T::native(D::Utf8)
}
fn real() -> T {
    T::native(D::Float64)
}
fn flag() -> T {
    T::native(D::Boolean)
}
fn ordinal() -> T {
    T::nonnegative(i64::from(u32::MAX))
}
fn record(fields: Vec<(&'static str, T)>) -> T {
    T::structure(fields.into_iter().map(|(n, t)| t.with_name(n)).collect())
}
fn port() -> T {
    record(vec![
        ("symbol_id", T::id()),
        ("quantity_id", T::id()),
        ("unit_id", T::id()),
    ])
}
pub(super) fn declare(b: &mut RegistryBuilder) {
    declare_dynamics_fitting(b);
    declare_balances(b);
    enumeration(
        b,
        "NativeVariableDomain",
        [
            "continuous",
            "integer",
            "binary",
            "semi_continuous",
            "semi_integer",
        ],
    );
    enumeration(b, "NativeObjectiveSense", ["minimize", "maximize"]);
    enumeration(
        b,
        "NativeMetricKind",
        ["real", "integer", "boolean", "text"],
    );
    let definition = record(vec![
        ("definition_id", T::id()),
        ("sources", T::list(text())),
        (
            "formals",
            T::list(record(vec![("path", text()), ("quantity_id", T::id())])),
        ),
        ("domains", T::list(text())),
        ("groups", T::list(text())),
        ("providers", T::list(text())),
        (
            "units",
            T::list(record(vec![("spelling", text()), ("unit_id", T::id())])),
        ),
        (
            "literals",
            T::list(record(vec![
                ("start", ordinal()),
                ("end", ordinal()),
                ("quantity_id", T::id()),
            ])),
        ),
    ]);
    let variable = record(vec![
        ("port", port()),
        ("fixed", flag()),
        ("domain", T::enumeration("NativeVariableDomain")),
        ("lower", real().optional()),
        ("upper", real().optional()),
    ]);
    let instance = record(vec![
        ("instance_id", T::id()),
        ("definition_id", T::id()),
        (
            "slots",
            T::list(record(vec![
                ("source_id", T::id()),
                ("formal_quantity_id", T::id()),
                ("formal_unit_id", T::id()),
            ])),
        ),
        (
            "contributions",
            T::list(record(vec![
                ("output", ordinal()),
                ("row_id", T::id().optional()),
                ("scale", real()),
            ])),
        ),
    ]);
    let case = record(vec![
        ("case_id", T::id()),
        ("name", text()),
        ("variables", T::list(variable)),
        ("parameters", T::list(port())),
        ("instances", T::list(instance)),
        (
            "rows",
            T::list(record(vec![
                ("row_id", T::id()),
                ("quantity_id", T::id()),
                ("lower", real().optional()),
                ("upper", real().optional()),
            ])),
        ),
        (
            "objective",
            record(vec![
                ("quantity_id", T::id()),
                ("sense", T::enumeration("NativeObjectiveSense")),
            ])
            .optional(),
        ),
        (
            "values",
            T::list(record(vec![("symbol_id", T::id()), ("value", real())])),
        ),
    ]);
    relation_version(
        b,
        N::Authored,
        "computation_models",
        2,
        S::Model,
        &["model_id"],
        vec![
            column("model_id", T::id()),
            column("name", text()),
            column("definitions", T::list(definition)),
            column(
                "domains",
                T::list(record(vec![
                    ("name", text()),
                    ("domain_id", T::id()),
                    ("members", T::list(T::id())),
                    ("kind", T::enumeration("DomainKind")),
                ])),
            ),
            column(
                "groups",
                T::list(record(vec![
                    ("name", text()),
                    ("quantity_id", T::id()),
                    ("axes", T::list(text())),
                    (
                        "slots",
                        T::list(record(vec![
                            ("members", T::list(T::id())),
                            ("slot", ordinal()),
                        ])),
                    ),
                ])),
            ),
            column("cases", T::list(case)),
        ],
        "Authoritative bounded native-model declaration. Typed builders and package documents share this contract; compiler products are derived and non-durable.",
    );
    relation(
        b,
        N::Runtime,
        "solve_runs",
        S::Derived,
        &["run_id", "step"],
        vec![
            column("run_id", T::id()),
            column("step", ordinal()),
            column("model_id", T::id()).optional(),
            column("revision", T::hash()).optional(),
            column("case_id", T::id()).optional(),
            column("backend", text()).optional(),
            column("native_code", T::native(D::Int64)).optional(),
            column("native_status", text()).optional(),
            column("termination", text()),
            column("assurance", text()),
            column("candidate_present", flag()),
            column("feasible", flag()).optional(),
            column("objective", real()).optional(),
            column("objective_sense", T::enumeration("NativeObjectiveSense")).optional(),
            column("objective_quantity_id", T::id()).optional(),
            column("validation_error", text()).optional(),
            column("error", text()).optional(),
            column("transformation", T::hash()).optional(),
        ],
        "Actual native termination, independent original-model validation and explicit unattempted/error states. No candidate implies no claimed solution.",
    );
    relation(
        b,
        N::Runtime,
        "solve_variables",
        S::Derived,
        &["run_id", "step", "symbol_id"],
        vec![
            column("run_id", T::id()),
            column("step", ordinal()),
            column("symbol_id", T::id()),
            column("quantity_id", T::id()).optional(),
            column("unit_id", T::id()).optional(),
            column("fixed", flag()),
            column("parameter", flag()),
            column("value", real()).optional(),
            column("lower", real()).optional(),
            column("upper", real()).optional(),
            column("lower_violation", real()).optional(),
            column("upper_violation", real()).optional(),
            column("tolerance", real()).optional(),
            column("lower_dual", real()).optional(),
            column("upper_dual", real()).optional(),
            column("reduced_cost", real()).optional(),
            column("stationarity", real()).optional(),
            column("dual_qualification", text()),
        ],
        "Original physical variable coordinates, including authored fixed values. Missing multipliers differ from zero. KKT residuals alone are not a sensitivity certificate.",
    );
    relation(
        b,
        N::Runtime,
        "solve_constraints",
        S::Derived,
        &["run_id", "step", "row_id"],
        vec![
            column("run_id", T::id()),
            column("step", ordinal()),
            column("row_id", T::id()),
            column("quantity_id", T::id()).optional(),
            column("unit_id", T::id()).optional(),
            column("value", real()).optional(),
            column("lower", real()).optional(),
            column("upper", real()).optional(),
            column("equality_residual", real()).optional(),
            column("lower_violation", real()).optional(),
            column("upper_violation", real()).optional(),
            column("tolerance", real()).optional(),
            column("dual", real()).optional(),
            column("dual_qualification", text()),
        ],
        "Fresh original constraint values; signed residual exists only for equality rows. Interval violations retain separate sides and physical tolerances.",
    );
    relation(
        b,
        N::Runtime,
        "solve_metrics",
        S::Derived,
        &["run_id", "step", "namespace", "name"],
        vec![
            column("run_id", T::id()),
            column("step", ordinal()),
            column("namespace", text()),
            column("name", text()),
            column("kind", T::enumeration("NativeMetricKind")),
            column("real", real()).optional(),
            column("integer", T::native(D::Int64)).optional(),
            column("boolean", flag()).optional(),
            column("text", text()).optional(),
        ],
        "Typed native metrics, effective options and provenance. Exactly the selected value field is populated by result admission; absent metrics are never synthesized as zero.",
    );
}

fn declare_dynamics_fitting(b: &mut RegistryBuilder) {
    relation_version(
        b,
        N::Authored,
        "native_providers",
        2,
        S::Model,
        &["model_id", "name"],
        vec![
            column("model_id", T::id()),
            column("name", text()),
            column("kind", text()),
            column("inputs", T::list(port())),
            column("outputs", T::list(port())),
            column(
                "envelope",
                record(vec![
                    ("temperature", T::list(real())),
                    ("density", T::list(real())),
                    ("pressure", T::list(real())),
                    ("composition", T::list(T::list(real()))),
                    ("provenance", text()),
                ]),
            ),
            column("caloric_reference", T::id()),
            column("components", T::list(T::id())),
            column("output", ordinal()),
        ],
        "Explicit native factory selection; currently feos-light-hydrocarbons. No Python callback or opaque provider state is persisted.",
    );
    let state = record(vec![
        ("symbol_id", T::id()),
        ("differential", flag()),
        ("initial_row", T::id()),
        ("offset", real()),
        ("scale", real()),
        ("residual_scale", real()),
    ]);
    let event = record(vec![
        ("event_id", T::id()),
        ("guard_row", T::id()),
        ("reset_rows", T::list(T::id())),
        ("terminal", flag()),
        ("next_mode", ordinal()),
        ("tolerance", real()),
    ]);
    relation(
        b,
        N::Authored,
        "dynamic_cases",
        S::Model,
        &["dynamic_id"],
        vec![
            column("dynamic_id", T::id()),
            column("model_id", T::id()),
            column("case_id", T::id()),
            column("time_id", T::id()),
            column("states", T::list(state)),
            column("parameters", T::list(T::id())),
            column("outputs", T::list(T::id())),
            column(
                "modes",
                T::list(record(vec![
                    ("rhs_rows", T::list(T::id())),
                    ("events", T::list(event)),
                ])),
            ),
        ],
        "Semi-explicit dynamics over existing compiled case functions. States use canonical physical offsets/scales; algebraic rows use explicit residual scales; time is seconds.",
    );
    relation(
        b,
        N::Authored,
        "fit_cases",
        S::Case,
        &["fit_id"],
        vec![
            column("fit_id", T::id()),
            column("model_id", T::id()),
            column(
                "parameters",
                T::list(record(vec![
                    ("symbol_id", T::id()),
                    ("fixed", flag()),
                    ("value", real()),
                    ("lower", real().optional()),
                    ("upper", real().optional()),
                    ("scale", real()),
                ])),
            ),
            column(
                "experiments",
                T::list(record(vec![
                    ("experiment_id", T::id()),
                    ("case_id", T::id()),
                    ("dynamic_id", T::id().optional()),
                ])),
            ),
            column(
                "observations",
                T::list(record(vec![
                    ("observation_id", T::id()),
                    ("experiment_id", T::id()),
                    ("output_id", T::id()),
                    ("time", real().optional()),
                    ("included", flag()),
                    ("importance", real()),
                ])),
            ),
        ],
        "Native simultaneous steady and smooth transient fitting. Measurement values, units and uncertainty remain authored.observations; elapsed time is seconds.",
    );
    relation(
        b,
        N::Runtime,
        "computation_runs",
        S::Derived,
        &["run_id"],
        vec![
            column("run_id", T::id()),
            column("kind", text()),
            column("source_identity", T::hash()),
            column("profile_identity", T::hash()),
            column("termination", text()),
            column("error", text()).optional(),
        ],
        "One joined native job, independent of the mathematical result representation.",
    );
    relation(
        b,
        N::Runtime,
        "simulation_samples",
        S::Derived,
        &["run_id", "sample", "symbol_id"],
        vec![
            column("run_id", T::id()),
            column("sample", ordinal()),
            column("time", real()),
            column("symbol_id", T::id()),
            column("quantity_id", T::id()),
            column("unit_id", T::id()),
            column("value", real()),
        ],
        "Successfully completed physical samples only; no missing trajectory tail is fabricated.",
    );
    relation(
        b,
        N::Runtime,
        "simulation_events",
        S::Derived,
        &["run_id", "ordinal", "symbol_id"],
        vec![
            column("run_id", T::id()),
            column("ordinal", ordinal()),
            column("event_id", T::id()).optional(),
            column("time", real()),
            column("symbol_id", T::id()),
            column("before", real()),
            column("after", real()).optional(),
        ],
        "Physical pre/post root and scheduled-input transitions. Absent after means a terminal or failed transition.",
    );
    relation(
        b,
        N::Runtime,
        "response_sensitivities",
        S::Derived,
        &[
            "run_id",
            "experiment_id",
            "sample",
            "output_id",
            "parameter_id",
        ],
        vec![
            column("run_id", T::id()),
            column("experiment_id", T::id()),
            column("sample", ordinal()),
            column("time", real()).optional(),
            column("output_id", T::id()),
            column("parameter_id", T::id()),
            column("output_unit_id", T::id()),
            column("parameter_unit_id", T::id()),
            column("value", real()),
        ],
        "Local physical response derivative in output difference units per parameter difference unit, never a confidence interval.",
    );
    relation(
        b,
        N::Runtime,
        "fit_variables",
        S::Derived,
        &["run_id", "experiment_id", "symbol_id"],
        vec![
            column("run_id", T::id()),
            column("experiment_id", T::id()),
            column("symbol_id", T::id()),
            column("quantity_id", T::id()),
            column("unit_id", T::id()),
            column("fixed", flag()),
            column("value", real()).optional(),
            column("lower", real()).optional(),
            column("upper", real()).optional(),
        ],
        "Original steady experiment state coordinates; no native alias is needed for physical interpretation.",
    );
    relation(
        b,
        N::Runtime,
        "fit_constraints",
        S::Derived,
        &["run_id", "experiment_id", "row_id"],
        vec![
            column("run_id", T::id()),
            column("experiment_id", T::id()),
            column("row_id", T::id()),
            column("quantity_id", T::id()),
            column("unit_id", T::id()),
            column("value", real()).optional(),
            column("lower", real()).optional(),
            column("upper", real()).optional(),
            column("tolerance", real()),
        ],
        "Independently evaluated original steady physical constraints and declared acceptance tolerances.",
    );
    relation(
        b,
        N::Runtime,
        "fit_parameters",
        S::Derived,
        &["run_id", "parameter_id"],
        vec![
            column("run_id", T::id()),
            column("parameter_id", T::id()),
            column("fixed", flag()),
            column("value", real()).optional(),
            column("unit_id", T::id()),
            column("scale", real()),
            column("at_bound", flag()).optional(),
        ],
        "Original parameter identities, fixed decisions and optional final candidate values.",
    );
    relation(
        b,
        N::Runtime,
        "fit_observations",
        S::Derived,
        &["run_id", "observation_id"],
        vec![
            column("run_id", T::id()),
            column("observation_id", T::id()),
            column("experiment_id", T::id()),
            column("included", flag()),
            column("prediction", real()).optional(),
            column("residual", real()).optional(),
            column("standardized_residual", real()).optional(),
            column("objective_contribution", real()).optional(),
            column("unit_id", T::id()),
        ],
        "Observation-aligned original physical predictions and half weighted squared standardized residual contributions.",
    );
}

fn declare_balances(b: &mut RegistryBuilder) {
    enumeration(
        b,
        "BalanceRole",
        [
            "inlet",
            "outlet",
            "generation",
            "consumption",
            "work_in",
            "work_out",
            "internal_in",
            "internal_out",
        ],
    );
    relation(
        b,
        N::Authored,
        "physical_balances",
        S::Model,
        &["balance_id"],
        vec![
            column("balance_id", T::id()),
            column("model_id", T::id()),
            column("case_id", T::id()),
            column("quantity_id", T::id()),
            column("accumulation", T::id()).optional(),
            column("tolerance", real()),
            column("integral_tolerance", real()).optional(),
            column("provenance", text()),
            column(
                "terms",
                T::list(record(vec![
                    ("source_id", T::id()),
                    ("role", T::enumeration("BalanceRole")),
                    ("transfer_id", T::id().optional()),
                    ("mode", ordinal().optional()),
                    ("instance_id", T::id()),
                    ("output", ordinal()),
                ])),
            ),
            column(
                "impulses",
                T::list(record(vec![("event_id", T::id()), ("value", real())])),
            ),
        ],
        "Authoritative signed physical contributions in canonical quantity coordinates. A balance derives one zero-equality steady row or one dynamic flux row. Accumulation and event impulses use the conserved state's canonical units. Declared tolerances and provenance are not empirical certification.",
    );
    relation(
        b,
        N::Runtime,
        "physical_checks",
        S::Derived,
        &["run_id", "step", "sample", "balance_id"],
        vec![
            column("run_id", T::id()),
            column("step", ordinal()),
            column("sample", ordinal()),
            column("balance_id", T::id()),
            column("quantity_id", T::id()),
            column("unit_id", T::id()),
            column("time", real()).optional(),
            column("closure", real()).optional(),
            column("tolerance", real()),
            column("accepted", flag()).optional(),
            column("error", text()).optional(),
            column("provenance", text()),
        ],
        "Independent contribution or accumulation-minus-integrated-flux closure, separate from native status and mathematical feasibility. Missing evaluation is not a pass.",
    );
}
