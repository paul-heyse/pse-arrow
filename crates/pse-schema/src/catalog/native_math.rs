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
    relation(
        b,
        N::Runtime,
        "solver_capabilities",
        S::Derived,
        &["backend"],
        vec![
            column("backend", T::enumeration("NativeBackend")),
            column("classes", T::list(T::enumeration("NativeProblemClass"))),
            column("derivatives", T::enumeration("NativeDerivativeCapability")),
            column("warm", T::enumeration("NativeWarmCapability")),
            column("reuse", text()),
            column("cancellation", text()),
            column("diagnostics", text()),
            column("general_bounds", flag()),
            column("sign_bounds", flag()),
            column("parallel", flag()),
        ],
        "Linked adapter inventory. Contextual eligibility is evaluated separately for the selected request.",
    );
    relation(
        b,
        N::Runtime,
        "run_lineage",
        S::Derived,
        &["run_id", "step"],
        vec![
            column("run_id", T::id()),
            column("step", ordinal()),
            column("model_id", T::id()),
            column("revision", T::hash()),
            column("case_id", T::id()),
            column("request_identity", T::hash()),
            column("preparation_identity", T::hash()),
            column("profile_identity", T::hash()),
            column("numerical_identity", T::hash()),
            column("physical_identity", T::hash()),
            column("environment_identity", T::hash()),
        ],
        "Completion-owned semantic lineage. Run identity names the attempt; it is not part of request identity. Effective settings, submitted start and native observations are retained in solve_metrics; source provider and parameter data are retained with the immutable revision.",
    );
    enumeration(b, "ComputationKind", ["simulation", "fit"]);
    enumeration(
        b,
        "TrajectoryTermination",
        [
            "completed",
            "event",
            "cancelled",
            "time_limit",
            "step_limit",
            "event_limit",
            "failed",
            "panic",
        ],
    );
    enumeration(
        b,
        "DualQualification",
        [
            "evaluated_kkt_not_sensitivity_certified",
            "unavailable_or_invalid",
            "unavailable",
            "not_applicable_parameter",
        ],
    );
    enumeration(
        b,
        "NumericalTarget",
        ["variable", "row", "objective", "observable", "closure"],
    );
    enumeration(b, "NumericalCoordinates", ["physical", "normalized"]);
    enumeration(
        b,
        "NumericalSource",
        [
            "analysis",
            "case",
            "model",
            "property_default",
            "quantity_nominal",
            "canonical_fallback",
        ],
    );
    enumeration(b, "ClosurePolicy", ["require_closed", "allow_unclosed"]);
    enumeration(
        b,
        "ClosureAssessment",
        ["not_required", "closed", "unclosed", "unavailable"],
    );
    enumeration(
        b,
        "CandidateUse",
        ["usable", "qualified_unclosed", "unusable"],
    );
    relation(
        b,
        N::Authored,
        "provider_scaling_bindings",
        S::Model,
        &["binding_id"],
        vec![
            column("binding_id", T::id()),
            column("model_id", T::id()),
            column("case_id", T::id()).optional(),
            column("provider", text()),
            column("output", ordinal()),
            column("target_id", T::id()),
            column("target_kind", T::enumeration("NumericalTarget")),
            column("property_package_id", T::id()),
            column("property_kind_id", T::id()),
            column("index", T::list(T::id())),
            column("provenance", text()),
        ],
        "Explicit provider output to numerical target and authored property default. No name, arity or package execution inference.",
    );
    relation(
        b,
        N::Runtime,
        "candidate_assessments",
        S::Derived,
        &["run_id", "step"],
        vec![
            column("run_id", T::id()),
            column("step", ordinal()),
            column("native_termination", T::enumeration("NativeTermination")).optional(),
            column("numerically_feasible", flag()).optional(),
            column("closure", T::enumeration("ClosureAssessment")),
            column("policy", T::enumeration("ClosurePolicy")),
            column("usability", T::enumeration("CandidateUse")),
            column("reason", text()),
        ],
        "Completion-owned candidate assessment. Native termination, original numerical acceptance, physical closure and final usability remain distinct.",
    );
    relation(
        b,
        N::Runtime,
        "resolved_numerics",
        S::Derived,
        &["run_id", "step", "target_kind", "target_id"],
        vec![
            column("run_id", T::id()),
            column("step", ordinal()),
            column("target_id", T::id()),
            column("target_kind", T::enumeration("NumericalTarget")),
            column("quantity_id", T::id()),
            column("unit_id", T::id()),
            column("nominal", real()),
            column("coordinate_scale", real()),
            column("absolute", real()),
            column("relative", real()),
            column("budget", real()),
            column(
                "provenance",
                T::list(record(vec![
                    ("declaration", T::id().optional()),
                    ("source", T::enumeration("NumericalSource")),
                    ("field", text()),
                    ("selected", flag()),
                    ("value", real()),
                    ("description", text()),
                ])),
            ),
        ],
        "Frozen original-representation budgets and selected/overridden source interpretations. Coordinate factors describe model normalization separately from native algorithmic scaling.",
    );
    relation(
        b,
        N::Authored,
        "numerical_requirements",
        S::Model,
        &["requirement_id"],
        vec![
            column("requirement_id", T::id()),
            column("model_id", T::id()),
            column("case_id", T::id()).optional(),
            column("target_id", T::id()),
            column("target_kind", T::enumeration("NumericalTarget")),
            column("nominal", real()).optional(),
            column("scaling_factor", real()).optional(),
            column("absolute_tolerance", real()).optional(),
            column("relative_tolerance", real()).optional(),
            column("unit_id", T::id()).optional(),
            column("coordinates", T::enumeration("NumericalCoordinates")),
            column("priority", T::native(D::Int32)),
            column("required", flag()),
            column("provenance", text()),
        ],
        "P05 declarative numerical meaning; selected ID targets, magnitude units and frozen relative budgets. Model/case selection establishes source precedence; runtime analysis overrides use the same row type.",
    );
    enumeration(
        b,
        "NativeBoundaryClass",
        [
            "invalid_model",
            "unsupported",
            "resource_limit",
            "trial_rejected",
            "nonfinite",
            "infrastructure",
            "cancelled",
            "conflict",
            "incompatible",
            "internal",
        ],
    );
    // Stable wire tags have one owner; native adapters implement behavior on these values.
    enumeration(
        b,
        "NativeBackend",
        [
            "ipopt", "pounce", "kinsol", "highs", "clarabel", "diffsol", "idas",
        ],
    );
    enumeration(
        b,
        "NativeTermination",
        [
            "success",
            "acceptable",
            "feasible_only",
            "infeasible",
            "unbounded",
            "infeasible_or_unbounded",
            "limit",
            "iteration_limit",
            "resource_exhausted",
            "inconclusive",
            "objective_limit",
            "solution_limit",
            "time_limit",
            "cancelled",
            "numerical",
            "evaluation",
            "panic",
            "invalid",
        ],
    );
    enumeration(
        b,
        "NativeStartPolicy",
        ["no_prior_start", "previous_accepted", "explicit"],
    );
    enumeration(
        b,
        "NativeQualification",
        [
            "unqualified",
            "feasible",
            "stationary",
            "optimal_within_tolerance",
            "gap_qualified",
        ],
    );
    enumeration(
        b,
        "NativeAssurance",
        [
            "none",
            "feasible",
            "local_stationary",
            "native_optimal",
            "certificate",
        ],
    );
    enumeration(
        b,
        "NativeRunState",
        ["native", "constant_evaluation", "rejected", "unattempted"],
    );
    enumeration(
        b,
        "NativeCandidateKind",
        [
            "final_iterate",
            "best_iterate",
            "feasible_point",
            "constant_evaluation",
        ],
    );
    enumeration(
        b,
        "EvidenceUnavailableReason",
        [
            "not_requested",
            "not_computed",
            "not_applicable",
            "unsupported",
            "failed",
            "unknown",
            "nonfinite",
        ],
    );
    enumeration(
        b,
        "TimeCoordinateKind",
        ["absolute_origin", "elapsed_duration"],
    );
    enumeration(
        b,
        "NativeProblemClass",
        [
            "smooth_nlp",
            "square_root",
            "declared_fixed_point",
            "linear",
            "mixed_linear",
            "convex_quadratic",
            "continuous_cone",
            "ode",
            "semi_explicit_index1",
        ],
    );
    enumeration(
        b,
        "NativeDerivativeCapability",
        [
            "exact_hessian_or_limited_memory",
            "jacobian_or_product",
            "coefficients",
            "first_with_smooth_sensitivities",
        ],
    );
    enumeration(
        b,
        "NativeWarmCapability",
        [
            "none",
            "primal",
            "primal_dual",
            "primal_dual_and_working_set",
            "primal_dual_and_basis",
        ],
    );
    declare_dynamics_fitting(b);
    declare_balances(b);
    relation(
        b,
        N::Authored,
        "model_compositions",
        S::Model,
        &["model_id"],
        vec![
            column("model_id", T::id()),
            column("root_instance_id", T::id()),
        ],
        "Selected root of a reusable template/instance composition. Lowering is a checked projection; authored templates remain authoritative.",
    );
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
        ["real", "integer", "boolean", "text", "unavailable"],
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
    relation_version(
        b,
        N::Runtime,
        "solve_runs",
        3,
        S::Derived,
        &["run_id", "step"],
        vec![
            column("run_id", T::id()),
            column("step", ordinal()),
            column("model_id", T::id()).optional(),
            column("revision", T::hash()).optional(),
            column("case_id", T::id()).optional(),
            column("backend", T::enumeration("NativeBackend")).optional(),
            column("native_code", T::native(D::Int64)).optional(),
            column("native_status", text()).optional(),
            column("state", T::enumeration("NativeRunState")),
            column("termination", T::enumeration("NativeTermination")).optional(),
            column("assurance", T::enumeration("NativeAssurance")),
            column("qualification", T::enumeration("NativeQualification")),
            column("candidate_kind", T::enumeration("NativeCandidateKind")).optional(),
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
    relation_version(
        b,
        N::Runtime,
        "solve_variables",
        2,
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
            column("dual_qualification", T::enumeration("DualQualification")),
        ],
        "Original physical variable coordinates, including authored fixed values. Missing multipliers differ from zero. KKT residuals alone are not a sensitivity certificate.",
    );
    relation_version(
        b,
        N::Runtime,
        "solve_constraints",
        2,
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
            column("dual_qualification", T::enumeration("DualQualification")),
        ],
        "Fresh original constraint values; signed residual exists only for equality rows. Interval violations retain separate sides and physical tolerances.",
    );
    relation_version(
        b,
        N::Runtime,
        "solve_metrics",
        2,
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
            column("unavailable", T::enumeration("EvidenceUnavailableReason")).optional(),
        ],
        "Typed native metrics, effective options and provenance. Exactly the selected value field is populated by result admission; absent metrics are never synthesized as zero.",
    );
}

fn declare_dynamics_fitting(b: &mut RegistryBuilder) {
    relation(
        b,
        N::Authored,
        "directional_valve_laws",
        S::Model,
        &["model_id", "name"],
        vec![
            column("model_id", T::id()),
            column("name", text()),
            column("transition_width_id", T::id()),
        ],
        "Declared nonreversing C2 valve closure: zero at nonpositive pressure difference, square-root law above the positive authored pressure transition width, and the unique quintic matching values and first two derivatives between. Width is a fixed positive pressure coordinate, never an implicit numerical tolerance.",
    );
    relation(
        b,
        N::Authored,
        "reaction_applications",
        S::Model,
        &["application_id"],
        vec![
            column("application_id", T::id()),
            column("model_id", T::id()),
            column("case_id", T::id()),
            column("material_system_id", T::id()),
            column("reaction_id", T::id()),
            column("phase_id", T::id()),
            column("rate_instance_id", T::id()),
            column("rate_output", ordinal()),
            column(
                "species_balances",
                T::list(record(vec![
                    ("species_id", T::id()),
                    ("balance_id", T::id()),
                ])),
            ),
            column("energy_balance_id", T::id()),
            column("heat_instance_id", T::id()),
            column("heat_output", ordinal()),
            column("element_tolerance", real()),
            column("provenance", text()),
        ],
        "Selected homogeneous molar reaction. The authored rate is an extent per time. Stoichiometry derives species source outputs; an explicit signed heat-rate output supplies energy. No formation energy or kinetics is guessed.",
    );
    enumeration(
        b,
        "ThermodynamicFormulation",
        ["homogeneous_density", "phase_equilibrium"],
    );
    enumeration(b, "StabilityPolicy", ["unchecked", "mechanical", "global"]);
    enumeration(
        b,
        "StabilityStatus",
        ["not_requested", "stable", "unstable", "failed"],
    );
    enumeration(b, "MissingInteractionPolicy", ["require_explicit", "zero"]);
    relation_version(
        b,
        N::Authored,
        "native_providers",
        3,
        S::Model,
        &["model_id", "name"],
        vec![
            column("model_id", T::id()),
            column("name", text()),
            column("kind", text()),
            column("material_system_id", T::id()).optional(),
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
            column("enthalpy_reference", T::id()),
            column("entropy_reference", T::id()),
            column(
                "components",
                T::list(record(vec![
                    ("species_id", T::id()),
                    ("pcsaft_cas", text()),
                    ("ideal_gas_cas", text()),
                ])),
            ),
            column("dependent_species", T::id()),
            column(
                "quantity_kinds",
                record(vec![
                    ("temperature", T::id()),
                    ("density", T::id()),
                    ("fraction", T::id()),
                    ("pressure", T::id()),
                    ("enthalpy", T::id()),
                    ("entropy", T::id()),
                    ("ln_fugacity", T::id()),
                ]),
            ),
            column(
                "data",
                record(vec![
                    ("pcsaft", text()),
                    ("ideal_gas", text()),
                    ("binary", text()),
                    ("provenance", text()),
                    (
                        "missing_interactions",
                        T::enumeration("MissingInteractionPolicy"),
                    ),
                ]),
            ),
            column("formulation", T::enumeration("ThermodynamicFormulation")),
            column("stability", T::enumeration("StabilityPolicy")),
            column("output", ordinal()),
        ],
        "Explicit PC-SAFT/DIPPR records, species mapping, physical roles and selected formulation. Independent composition coordinates follow component order with the declared dependent species omitted; outputs are pressure, enthalpy, entropy and ordered ln(phi). No Python callback or opaque provider state is persisted.",
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
    enumeration(b, "ObservationTimeBasis", ["elapsed", "model_clock"]);
    relation_version(
        b,
        N::Authored,
        "dynamic_cases",
        2,
        S::Model,
        &["dynamic_id"],
        vec![
            column("dynamic_id", T::id()),
            column("model_id", T::id()),
            column("case_id", T::id()),
            column("time_id", T::id()),
            column("time_origin", real()).optional(),
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
        "Semi-explicit dynamics over existing compiled functions. Integration time is canonical seconds; model time is (integration time - time_origin) divided by the time port unit scale. An absent origin is zero.",
    );
    relation_version(
        b,
        N::Authored,
        "fit_cases",
        2,
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
                    (
                        "time_basis",
                        T::enumeration("ObservationTimeBasis").optional(),
                    ),
                    ("time_unit_id", T::id().optional()),
                    ("included", flag()),
                    ("importance", real()),
                ])),
            ),
        ],
        "Native simultaneous fitting. Observation time defaults to elapsed seconds from the integration start; model_clock uses the declared dynamic time origin. Explicit time units must be non-affine time units. Measurement values, units and uncertainty remain authored.observations.",
    );
    relation_version(
        b,
        N::Runtime,
        "computation_runs",
        2,
        S::Derived,
        &["run_id"],
        vec![
            column("run_id", T::id()),
            column("kind", T::enumeration("ComputationKind")),
            column("source_identity", T::hash()),
            column("profile_identity", T::hash()),
            column("state", T::enumeration("NativeRunState")),
            column("termination", T::enumeration("NativeTermination")).optional(),
            column(
                "trajectory_termination",
                T::enumeration("TrajectoryTermination"),
            )
            .optional(),
            column("backend", T::enumeration("NativeBackend")).optional(),
            column("native_code", T::native(D::Int64)).optional(),
            column("native_status", text()).optional(),
            column("qualification", T::enumeration("NativeQualification")),
            column("candidate_kind", T::enumeration("NativeCandidateKind")).optional(),
            column("candidate_available", flag()),
            column("feasible", flag()).optional(),
            column("completed_time", real()).optional(),
            column("completed_samples", ordinal()).optional(),
            column("estimate_qualified", flag()).optional(),
            column("response_available", flag()).optional(),
            column("response_rank", ordinal()).optional(),
            column("response_condition", real()).optional(),
            column("validation_error", text()).optional(),
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
            "heat_in",
            "heat_out",
            "work_in",
            "work_out",
            "internal_in",
            "internal_out",
        ],
    );
    relation_version(
        b,
        N::Authored,
        "physical_balances",
        2,
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
                    ("multiplier", real()),
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
