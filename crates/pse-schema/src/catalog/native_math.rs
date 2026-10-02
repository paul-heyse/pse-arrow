// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durable declarations and physical results for the native library compiler.
//! Expressions are authored DSL; no CAS serialization or evaluator state is durable.
use super::declarations::{
    column, documented, enumeration, identity, relation, relation_version, run_id,
};
use crate::{
    RegistryBuilder,
    model::{EnumDecl, EnumMember, FieldContract as T, Namespace as N, SnapshotClass as S},
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
/// The model a row belongs to: the specialized definition, typed `declaration`. Its
/// derivations live in `pse_model::lineage`; these column documents own the meaning.
fn model_id(doc: &'static str) -> T {
    documented("model_id", T::id(), doc).with_identity("declaration")
}
/// The case a row belongs to: the solved root when it is a case, typed `declaration`.
fn case_id(doc: &'static str) -> T {
    documented("case_id", T::id(), doc)
        .with_identity("declaration")
        .optional()
}
/// The instance a row's solve prepared, typed `instance`.
fn instance_id(doc: &'static str) -> T {
    documented("instance_id", T::id(), doc)
        .with_identity("instance")
        .optional()
}
/// The fit a row belongs to, typed `fit`.
fn fit_id(doc: &'static str) -> T {
    documented("fit_id", T::id(), doc)
        .with_identity("fit")
        .optional()
}
pub(super) fn declare(b: &mut RegistryBuilder) {
    relation_version(
        b,
        N::Runtime,
        "solver_capabilities",
        4,
        S::Derived,
        &["backend"],
        vec![
            column("backend", T::enumeration("NativeBackend")),
            column(
                "structural_policy",
                T::enumeration("NativeStructuralPolicy"),
            ),
            column(
                "lexicographic_degradation",
                T::enumeration("NativeLexicographicDegradation"),
            ),
            column("classes", T::list(T::enumeration("NativeProblemClass"))),
            column(
                "automatic_classes",
                T::list(T::enumeration("NativeProblemClass")),
            ),
            column("derivatives", T::enumeration("NativeDerivativeCapability")),
            column("warm", T::enumeration("NativeWarmCapability")),
            column("reuse", text()),
            column("cancellation", text()),
            column("diagnostics", text()),
            column("general_bounds", flag()),
            column("sign_bounds", flag()),
            column("parallel", flag()),
            column("certifies", flag()),
            column(
                "native_forms",
                T::list(T::enumeration("NativeConstraintForm")),
            ),
            column(
                "requirements",
                T::list(T::enumeration("ModelingStructuralRequirement")),
            ),
            column(
                "lexicographic_classes",
                T::list(T::enumeration("NativeProblemClass")),
            ),
            column("batch", flag()),
            column("sensitivities", flag()),
        ],
        "Linked adapter inventory. Contextual eligibility is evaluated separately for the selected request. `automatic_classes` are the classes automatic routing may choose the adapter for; every other class in `classes` needs explicit selection. Automatic routing takes the problem's classes most specific first and selects among the eligible adapters automatic for the first class that has one. `certifies` marks an adapter that serves the explicit certify intent with global_bound and proven_infeasible assurances. `native_forms` lists the constraint handlers the adapter consumes; a structure that leaves any other form to a native handler is ineligible (ADR-0104). `requirements` lists the structural requirements of a formulation the adapter can honour with a method its settings select, such as the l1 exact penalty an authored `penalty(l1)` realization states; a structure stating any other is ineligible, as is a request whose settings select another method (ADR-0104 §5). `lexicographic_classes` lists the classes in which the adapter optimizes several objectives lexicographically in one native solve; a structure with several objectives in any other class is ineligible (ADR-0111). `batch` marks an adapter that solves the independent points of a study sharing one prepared structure as one parallel batch on the admitted threads, each point still qualified on its own (Plan 22 N5). `sensitivities` marks an adapter supporting contextual parameter analysis: original-coordinate multipliers for optimizing KKT sensitivity, or qualified regular-square Root response (ADR-0144); automatic routing of a request for parametric sensitivities prefers one, and an explicit selection of any other solves with the quantities withheld and their reason recorded (ADR-0118).",
    );
    relation_version(
        b,
        N::Runtime,
        "run_lineage",
        2,
        S::Derived,
        &["run_id", "step"],
        vec![
            run_id(),
            column("step", ordinal()),
            model_id(
                "The specialized definition the step solved: the root declaration its model specializes. A fit names the one definition all its experiments specialize, and none when they specialize different ones.",
            )
            .optional(),
            column("revision", T::hash()),
            case_id(
                "The case declaration the step solved: its root when that root is a case or a test (a case with an oracle), otherwise absent. A fit names the one case all its experiments solve, if they share one.",
            ),
            instance_id(
                "The instance the step's root became: the root declaration's own identity for an ordinary solve or simulation, the experiment's instance for a fit experiment. Absent for a fit, which spans its experiments' instances.",
            ),
            fit_id("The fit the step solved; absent for a modeling solve or simulation."),
            column("request_identity", T::hash()),
            column("preparation_identity", T::hash()),
            column("profile_identity", T::hash()),
            column("numerical_identity", T::hash()),
            column("physical_identity", T::hash()),
            column("environment_identity", T::hash()),
        ],
        "Completion-owned semantic lineage. The run is the execution, not its content: it is not part of request identity, and a durable run's tries are attempts recorded in the operational store, the publication naming the attempt. Model, case, instance and fit name what was solved (`pse_model::lineage`). Effective settings, submitted start and native observations are retained in solve_metrics; source provider and parameter data are retained with the immutable revision.",
    );
    enumeration(b, "ComputationKind", ["simulation", "fit", "shooting"]);
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
            "sensitivity_certified",
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
            "model_hint",
            "property_default",
            "derived_nominal",
            "quantity_nominal",
            "canonical_fallback",
        ],
    );
    // The numerical field a provenance entry resolved (ADR-0115 Outcome 3).
    enumeration(
        b,
        "NumericalProvenanceField",
        [
            "nominal",
            "absolute_tolerance",
            "relative_tolerance",
            "coordinate_scale",
        ],
    );
    enumeration(
        b,
        "NativeStructuralMode",
        ["roots", "nlp", "native_feasibility", "point_evaluation"],
    );
    enumeration(
        b,
        "NativeRepresentation",
        [
            "nlp",
            "roots",
            "coefficients",
            "cone",
            "factorable",
            "trajectory",
        ],
    );
    enumeration(b, "NativeRouteSelection", ["auto", "explicit"]);
    enumeration(b, "NativeRouteKind", ["constant", "native"]);
    enumeration(b, "NativeLexicographicRealization", ["native", "staged"]);
    enumeration(
        b,
        "NativeRouteRefusal",
        [
            "invalid_request",
            "no_eligible",
            "unavailable",
            "ineligible",
            "constant_native_forms",
            "structure",
        ],
    );
    enumeration(
        b,
        "StructuralScopeKind",
        ["whole", "independent", "conditional", "partial"],
    );
    relation(
        b,
        N::Runtime,
        "route_decisions",
        S::Derived,
        &["request_identity", "step"],
        vec![
            column("request_identity", T::hash()),
            column("step", ordinal()),
            column("intent", T::enumeration("NativeSolveIntent")),
            column("selection", T::enumeration("NativeRouteSelection")),
            column("requested_backend", T::enumeration("NativeBackend")).optional(),
            column("classes", T::list(T::enumeration("NativeProblemClass"))),
            column(
                "eligibility",
                T::list(record(vec![
                    ("backend", T::enumeration("NativeBackend")),
                    ("reasons", T::list(T::enumeration("NativeIneligibility"))),
                ])),
            ),
            column("selected", T::enumeration("NativeRouteKind")).optional(),
            column("backend", T::enumeration("NativeBackend")).optional(),
            column("representation", T::enumeration("NativeRepresentation")).optional(),
            column(
                "lexicographic",
                T::enumeration("NativeLexicographicRealization"),
            )
            .optional(),
            column("refusal", T::enumeration("NativeRouteRefusal")).optional(),
            column("detail", text()).optional(),
        ],
        "Request-qualified capability admission retained even without a native attempt. No failed route fabricates a run or solver result.",
    );
    relation(
        b,
        N::Runtime,
        "structural_assessments",
        S::Derived,
        &["request_identity", "step"],
        vec![
            column("request_identity", T::hash()),
            column("step", ordinal()),
            column("mode", T::enumeration("NativeStructuralMode")),
            column("scope", T::enumeration("StructuralScopeKind")),
            column("scope_model", T::id()),
            column("scope_members", T::list(T::id())),
            column("scope_rows", T::list(T::id())),
            column("scope_columns", T::list(T::id())),
            column("scope_inputs", T::list(T::id())),
            column("variables", T::list(T::id())),
            column(
                "equations",
                T::list(record(vec![
                    ("id", T::id()),
                    ("lower", real().optional()),
                    ("upper", real().optional()),
                ])),
            ),
            column(
                "matching",
                T::list(record(vec![("row", T::id()), ("column", T::id())])),
            ),
            column("unmatched_rows", T::list(T::id())),
            column("unmatched_columns", T::list(T::id())),
            column("optimization_freedom", ordinal()).optional(),
            column("admitted", flag()),
            column("provenance", text()),
        ],
        "Complete original bound-view structural assessment with library matching and named refusals; modes distinguish Root matching, equality matching, justified native feasibility and constant point evaluation. No numerical rank claim.",
    );
    enumeration(
        b,
        "NativeStructuralPolicy",
        ["roots", "equalities", "native_feasibility", "factorable"],
    );
    enumeration(
        b,
        "NativeLexicographicDegradation",
        ["max", "single_nonzero"],
    );
    enumeration(
        b,
        "EndpointPolicy",
        ["fixed_horizon", "declared_terminal_event"],
    );
    enumeration(
        b,
        "IncumbentPolicy",
        ["refuse", "accept_feasible", "accept_within_gap"],
    );
    enumeration(
        b,
        "CandidateQualifier",
        [
            "accepted_incumbent_feasible",
            "accepted_incumbent_within_gap",
            "closure_allowed",
            "applicability_unknown_allowed",
            "applicability_extrapolation_allowed",
        ],
    );
    enumeration(
        b,
        "CandidateRefusal",
        [
            "no_candidate",
            "validation_failed",
            "infeasible",
            "unqualified",
            "native_outcome",
            "model_checks",
            "closure_unavailable",
            "closure_unclosed",
            "applicability_unavailable",
            "applicability_denied",
            "endpoint_unavailable",
            "coverage_unavailable",
            "bound_unavailable",
            "gap_exceeded",
            "least_infeasible",
            "relaxed_incumbent",
            "incumbent_refused",
        ],
    );
    enumeration(
        b,
        "CandidateBoundOrigin",
        [
            "global_export",
            "linear_program",
            "mixed_integer",
            "conic_program",
        ],
    );
    enumeration(b, "ClosurePolicy", ["require_closed", "allow_unclosed"]);
    enumeration(
        b,
        "ClosureAssessment",
        ["not_required", "closed", "unclosed", "unavailable"],
    );
    // ADR-0106: one candidate-use decision, owned by the workflow completion owner.
    enumeration(
        b,
        "CandidateUse",
        [
            "usable",
            "qualified_unclosed",
            "seed_only",
            "diagnostic_only",
            "unusable",
        ],
    );
    relation_version(
        b,
        N::Runtime,
        "candidate_assessments",
        3,
        S::Derived,
        &["run_id", "step"],
        vec![
            run_id(),
            column("step", ordinal()),
            column("native_termination", T::enumeration("NativeTermination")).optional(),
            column("numerically_feasible", flag()).optional(),
            column("closure", T::enumeration("ClosureAssessment")),
            column("policy", T::enumeration("ClosurePolicy")),
            column("usability", T::enumeration("CandidateUse")),
            column("reason", text()),
            column("incumbent_policy", T::enumeration("IncumbentPolicy")),
            column("candidate_kind", T::enumeration("NativeCandidateKind")).optional(),
            column("qualification", T::enumeration("NativeQualification")).optional(),
            column("validated", flag()).optional(),
            column("bound_origin", T::enumeration("CandidateBoundOrigin")).optional(),
            column("bound", real()).optional(),
            column("absolute_gap", real()).optional(),
            column("relative_gap", real()).optional(),
            column("qualifiers", T::list(T::enumeration("CandidateQualifier"))),
            column("refusals", T::list(T::enumeration("CandidateRefusal"))),
            column("permits_result", flag()),
            column("permits_seed", flag()),
        ],
        "Completion-owned candidate assessment. Native termination, original numerical acceptance, physical closure and final usability remain distinct. A seed-only candidate may seed a later step and is never a published solution; a diagnostic-only point is an observation only.",
    );
    relation(
        b,
        N::Runtime,
        "resolved_numerics",
        S::Derived,
        &["run_id", "step", "target_kind", "target_id"],
        vec![
            run_id(),
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
                    ("field", T::enumeration("NumericalProvenanceField")),
                    ("selected", flag()),
                    ("value", real()),
                    ("description", text()),
                ])),
            ),
        ],
        "Frozen original-representation budgets and selected/overridden source interpretations. Coordinate factors describe model normalization separately from native algorithmic scaling.",
    );
    relation_version(
        b,
        N::Authored,
        "numerical_requirements",
        2,
        S::Model,
        &["requirement_id"],
        vec![
            column("requirement_id", T::id()),
            model_id(
                "The specialized definition the requirement belongs to: the root declaration of the model it is declared for. A modeling preparation's requirements name its root, an implicit block's trial hints their enclosing model's; a fit's parameter requirements name the one definition all its experiments specialize, and none when they specialize different ones.",
            )
            .optional(),
            case_id(
                "The case declaration the requirement belongs to: the solved root when it is a case or a test, otherwise absent.",
            ),
            instance_id(
                "The instance whose preparation declared the requirement: the solved root instance (a fit experiment's own instance), or the implicit block whose trial hints declared it. Absent for a fit's parameter requirements and a generated rate system's.",
            ),
            fit_id("The fit whose parameter requirement this is; absent otherwise."),
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
    // ADR-0106 (DP-21): algorithmic numerical failure and inconclusive outcomes are
    // classes of their own; every diagnostic also carries a severity.
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
            "numerical",
            "inconclusive",
        ],
    );
    enumeration(b, "DiagnosticSeverity", ["error", "warning", "info"]);
    // Stable wire tags have one owner; native adapters implement behavior on these values.
    enumeration(
        b,
        "NativeBackend",
        [
            "ipopt",
            "pounce",
            "kinsol",
            "highs",
            "clarabel",
            "diffsol",
            "idas",
            "scip",
            "pounce_convex",
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
            "node_limit",
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
    // Mathematical purpose of a request; `certify` is only ever selected explicitly (ADR-0106).
    enumeration(
        b,
        "NativeSolveIntent",
        [
            "optimize",
            "root",
            "feasible_point",
            "initialize",
            "certify",
        ],
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
    // Each assurance states its conditions; only `exact_certificate` is rigorous (ADR-0106 §9).
    b.declare_enum(EnumDecl::platform(
        "NativeAssurance",
        vec![
            EnumMember::new("none", "No claim beyond the native termination."),
            EnumMember::new(
                "feasible",
                "The candidate meets every original-coordinate tolerance.",
            ),
            EnumMember::new(
                "local_stationary",
                "Original-coordinate KKT conditions hold within the resolved budgets; a local claim.",
            ),
            EnumMember::new(
                "native_optimal",
                "The native method's optimality or gap test holds within its tolerances.",
            ),
            EnumMember::new(
                "certificate",
                "A native infeasibility or unboundedness certificate, qualified by its residuals.",
            ),
            EnumMember::new(
                "global_bound",
                "A dual bound on the exported program over the declared box, valid within the backend's recorded feasibility and optimality tolerances and the export fidelity; not interval-rigorous.",
            ),
            EnumMember::new(
                "proven_infeasible",
                "The backend's global infeasibility conclusion for the exported program over the declared box, under the conditions of global_bound; a relaxed export keeps it sound; not interval-rigorous.",
            ),
            EnumMember::new(
                "exact_certificate",
                "Optimality or infeasibility established in rational arithmetic; the only rigorous assurance.",
            ),
            EnumMember::new(
                "sos_bound_nonrigorous",
                "A floating-point sum-of-squares polynomial lower bound; never a certificate and never part of a gap claim.",
            ),
        ],
    ));
    // Plan 23 H10 (PS-10): a backend's infeasibility conclusion is checked against every
    // point known for the same problem; the witness that contradicts it names its source.
    b.declare_enum(EnumDecl::platform(
        "NativeInfeasibilityWitness",
        vec![
            EnumMember::new(
                "candidate",
                "The point the backend reported beside its infeasibility conclusion.",
            ),
            EnumMember::new("pool", "A solution the backend stored in its solution pool."),
            EnumMember::new(
                "incumbent",
                "An incumbent the backend reported while it searched.",
            ),
            EnumMember::new("seed", "The primal warm start submitted to the backend."),
            EnumMember::new("start", "The case's start values."),
            EnumMember::new(
                "local_solution",
                "The candidate of a local solve of the same problem from the start, run through the one NLP runner.",
            ),
        ],
    ));
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
            "nonconvex_quadratic",
            "mixed_integer_quadratic",
            "mixed_integer_nonlinear",
        ],
    );
    enumeration(
        b,
        "NativeDerivativeCapability",
        [
            "exact_hessian_or_limited_memory",
            "jacobian_or_product",
            "coefficients",
            "forward_and_adjoint_sensitivities",
            "second_order_adjoint_sensitivities",
            "factorable",
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
    // Why one adapter cannot represent a request: the typed eligibility reason code that
    // crosses the Python boundary (ADR-0113 §3).
    enumeration(
        b,
        "NativeIneligibility",
        [
            "not_linked",
            "serial",
            "not_square_root",
            "no_objective",
            "certification",
            "class",
            "derivatives",
            "bounds",
            "native_forms",
            "least_squares",
            "method",
            "lexicographic",
        ],
    );
    // Constraint handlers a native realization leaves to the backend (ADR-0104).
    enumeration(
        b,
        "NativeConstraintForm",
        [
            "indicator",
            "sos1",
            "sos2",
            "and",
            "or",
            "xor",
            "cardinality",
        ],
    );
    declare_dynamics_fitting(b);
    declare_local_analysis(b);
    enumeration(b, "NativeObjectiveSense", ["minimize", "maximize"]);
    enumeration(
        b,
        "NativeMetricKind",
        ["real", "integer", "boolean", "text", "unavailable"],
    );
    relation_version(
        b,
        N::Runtime,
        "solve_runs",
        5,
        S::Derived,
        &["run_id", "step"],
        vec![
            run_id(),
            column("step", ordinal()),
            model_id("The specialized definition the step solved: the root declaration its model specializes.")
                .optional(),
            column("revision", T::hash()).optional(),
            case_id(
                "The case declaration the step solved: its root when that root is a case or a test, otherwise absent.",
            ),
            instance_id(
                "The instance the step's root became: the root declaration's own identity, or a fit experiment's instance.",
            ),
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
            column(
                "commitment",
                T::list(record(vec![
                    ("source_id", T::id()),
                    ("lower", real()),
                    ("upper", real()),
                ])),
            )
            .optional(),
        ],
        "Actual native termination, independent original-model validation and explicit unattempted/error states. No candidate implies no claimed solution. `commitment` states the discrete assignment that the step's multipliers and every quantity derived from them are conditional on (ADR-0118 item 9), from the SCIP fixed-assignment re-solve or the HiGHS fixed-commitment LP: each committed column with its closed box in original coordinates, degenerate for a fixed value (an integer value or a semi column's zero branch) and the active interval for a semicontinuous column on its active branch. It is absent when the multipliers are not conditional on an assignment.",
    );
    relation_version(
        b,
        N::Runtime,
        "solve_variables",
        3,
        S::Derived,
        &["run_id", "step", "symbol_id"],
        vec![
            run_id(),
            column("step", ordinal()),
            column("symbol_id", T::id()),
            column("quantity_id", T::id()).optional(),
            column("unit_id", T::id()).optional(),
            column("fixed", flag()),
            column("parameter", flag()),
            column("domain", T::enumeration("ModelingVariableDomain")).optional(),
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
        "Original physical variable coordinates, including authored fixed values. Missing multipliers differ from zero. KKT residuals alone are not a sensitivity certificate. Every variable row states its declared domain (ADR-0103); parameter rows carry none.",
    );
    relation_version(
        b,
        N::Runtime,
        "solve_constraints",
        2,
        S::Derived,
        &["run_id", "step", "row_id"],
        vec![
            run_id(),
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
            run_id(),
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
    relation_version(
        b,
        N::Runtime,
        "solution_pool",
        1,
        S::Derived,
        &["run_id", "step", "rank", "symbol_id"],
        vec![
            run_id(),
            column("step", ordinal()),
            column("rank", ordinal()),
            column("symbol_id", T::id()),
            column("value", real()),
            column("objective", real()).optional(),
            column("feasible", flag()).optional(),
        ],
        "Ranked solutions a certifying backend stored beside the candidate, best first, over the free variables in original coordinates (ADR-0105 §8). `objective` is the backend's value in the authored sense; `feasible` is the original-coordinate re-qualification, absent when the point could not be evaluated. A pooled solution is an observation: only the qualified candidate is a result or a seed.",
    );
    relation_version(
        b,
        N::Runtime,
        "incumbents",
        1,
        S::Derived,
        &["run_id", "seq"],
        vec![
            run_id(),
            column("seq", ordinal()),
            column("step", ordinal()),
            column("elapsed_seconds", real()),
            column("phase", text()),
            column("objective", real()),
            column("dual_bound", real()).optional(),
            column("gap", real()).optional(),
            column("nodes", T::native(D::Int64)).optional(),
            column("seconds", real()).optional(),
            column("solution_id", T::id()).optional(),
        ],
        "The incumbent stream of a durable run as the operational store holds it when the attempt ends (Plan 22 I13): each improving feasible point of a branch-and-bound search with the bound at that time, numbered `seq` by the producer, with the step, the phase and `elapsed_seconds` of the event that reported it. `solution_id` names the captured point stored for resumption; the store prunes captures with their stream, so the published row outlives it. An ephemeral run keeps its retained incumbents in runtime.solve_metrics instead.",
    );
    declare_certificates(b);
}

/// Typed infeasibility and unboundedness certificates in original coordinates (Plan 22 I11).
fn declare_certificates(b: &mut RegistryBuilder) {
    b.declare_enum(EnumDecl::platform(
        "NativeCertificateKind",
        vec![
            EnumMember::new(
                "primal_infeasible",
                "A Farkas ray y over the cone rows: Aᵀy = 0, bᵀy < 0 and y in the dual cone prove that no point satisfies the constraints.",
            ),
            EnumMember::new(
                "dual_infeasible",
                "A recession direction x: Px = 0, -Ax in the cone and qᵀx < 0 prove the dual infeasible: the objective is unbounded below whenever the constraints admit a point.",
            ),
        ],
    ));
    b.declare_enum(EnumDecl::platform(
        "NativeCertificateAccuracy",
        vec![
            EnumMember::new(
                "full",
                "The native method met its full infeasibility tolerances.",
            ),
            EnumMember::new(
                "reduced",
                "The native method met only its reduced (almost) tolerances; never a certificate, whatever its verification.",
            ),
        ],
    ));
    b.declare_enum(EnumDecl::platform(
        "NativeRayCoordinate",
        vec![
            EnumMember::new(
                "row",
                "The cone row of a row identity: an explicit cone row, or the zero-cone row of an equality.",
            ),
            EnumMember::new(
                "row_lower",
                "The nonnegative row -a·x + s = -L of a row's finite lower bound.",
            ),
            EnumMember::new(
                "row_upper",
                "The nonnegative row a·x + s = U of a row's finite upper bound.",
            ),
            EnumMember::new(
                "variable_lower",
                "The nonnegative row -x + s = -l of a variable's finite lower bound.",
            ),
            EnumMember::new(
                "variable_upper",
                "The nonnegative row x + s = u of a variable's finite upper bound.",
            ),
            EnumMember::new("variable", "A variable of a recession direction."),
        ],
    ));
    relation(
        b,
        N::Runtime,
        "infeasibility_certificates",
        S::Derived,
        &["run_id", "step"],
        vec![
            run_id(),
            column("step", ordinal()),
            column("backend", T::enumeration("NativeBackend")),
            column("kind", T::enumeration("NativeCertificateKind")),
            column("accuracy", T::enumeration("NativeCertificateAccuracy")),
            column(
                "ray",
                T::list(record(vec![
                    ("coordinate", T::enumeration("NativeRayCoordinate")),
                    ("source_id", T::id()),
                    ("value", real()),
                ])),
            ),
            column(
                "verification",
                record(vec![
                    ("residual", real()),
                    ("objective", real()),
                    ("cone", real()),
                    ("margin", real()),
                    ("tolerance", real()),
                    ("verified", flag()),
                ]),
            )
            .optional(),
        ],
        "A native infeasibility (Farkas) or unboundedness ray, in original physical coordinates over the cone form of the solved problem: its rows (a cone row, or a coefficient row's lower and upper sides) then its finite variable bounds, lower before upper. Verification recomputes it against the original data: `residual` is the worst column's |Aᵀy| (or row's |Px|, |Ax| on zero rows) relative to the magnitudes it sums, `objective` is bᵀy (or qᵀx) and `cone` the dual-cone (or cone) violation, both relative to the ray's largest entry; `margin` is -bᵀy less the rows' and bounds' acceptance budgets weighted by |y| (or -qᵀx), relative to the same entry. A ray is verified when every relative quantity is within `tolerance` and the margin is positive, so a problem infeasible by less than its acceptance budgets is never certified. Only a verified ray at full accuracy carries the certificate assurance. Absent verification means the original data could not be evaluated.",
    );
}

fn declare_dynamics_fitting(b: &mut RegistryBuilder) {
    enumeration(b, "ObservationTimeBasis", ["elapsed", "model_clock"]);
    identity(
        b,
        "fit",
        "One authored shared-parameter fit over its experiments",
    );
    // An experiment prepares its case under its own instance identity, so two experiments
    // of one case are two instances: experiment columns carry `instance`, and an
    // experiment's case is the root declaration it prepares.
    relation_version(
        b,
        N::Authored,
        "fit_cases",
        4,
        S::Case,
        &["fit_id"],
        vec![
            column("fit_id", T::id()).with_owned_identity("fit"),
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
                    ("experiment_id", T::id().with_identity("instance")),
                    ("case_id", T::id().with_identity("declaration")),
                    ("route", T::enumeration("ModelingAnalysisRoute")),
                    (
                        "bindings",
                        T::list(record(vec![("parameter_id", T::id()), ("path", text())])),
                    ),
                ])),
            ),
            column(
                "observations",
                T::list(record(vec![
                    ("observation_id", T::id().with_identity("declaration")),
                    ("value_attribute", text()),
                    ("standard_deviation_attribute", text().optional()),
                    ("experiment_id", T::id().with_identity("instance")),
                    ("output_path", text()),
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
        "Shared-parameter fitting over authored modeling cases. Each experiment binds shared parameter identities to local source paths in canonical physical units. Observation paths select original members. Elapsed time is relative to the integration start; model_clock is the authored axis coordinate. Measurement values, full quantity conventions, uncertainty and provenance are selected from admitted typed record attributes; fit declarations carry only experiment and attribute selection intent.",
    );
    relation(
        b,
        N::Runtime,
        "fitted_parameter_cells",
        S::Derived,
        &["run_id", "parameter_id"],
        vec![
            run_id(),
            column("fit_id", T::id().with_identity("fit")),
            column("source_revision", T::hash()),
            column("fit_source", T::hash()),
            column("parameter_id", T::id()),
            column("quantity_type_id", T::id()),
            column("unit_id", T::id()),
            column("value", real()),
        ],
        "Explicit export from a freshly qualified, locally identifiable fit. Values use canonical units and retain their full physical quantity convention and producing fit, run and source revision. This read-only export never mutates a parameter bank; an authored fitted set names its fit receipt through fit lineage.",
    );
    relation_version(
        b,
        N::Runtime,
        "computation_runs",
        2,
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
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
            run_id(),
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
        "trajectory_endpoints",
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
            column("requirement", T::enumeration("EndpointPolicy")),
            column("required_event", T::id()).optional(),
            column("event_id", T::id()).optional(),
            column("time", real()),
            column("mode", ordinal()),
            column("state_ids", T::list(T::id())),
            column("input_ids", T::list(T::id())),
            column("output_ids", T::list(T::id())),
            column("state", T::list(real())),
            column("outputs", T::list(real())),
            column("integrals", T::list(real())),
            column("input_columns", T::list(ordinal())),
            column("inputs", T::list(real())),
            column("endpoint_satisfied", flag()),
            column("prefix_complete", flag()),
            column("missing_observations", T::list(real())),
        ],
        "Actual completion state and inputs in canonical physical coordinates, with stable coordinate IDs, and live pre-termination input segment, separate from requested samples. Future required observations remain explicitly missing; fixed-domain integrals retain their extent.",
    );
    relation(
        b,
        N::Runtime,
        "simulation_events",
        S::Derived,
        &["run_id", "ordinal", "symbol_id"],
        vec![
            run_id(),
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
            run_id(),
            column("experiment_id", T::id()).with_identity("instance"),
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
            run_id(),
            column("experiment_id", T::id()).with_identity("instance"),
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
            run_id(),
            column("experiment_id", T::id()).with_identity("instance"),
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
            run_id(),
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
            run_id(),
            column("observation_id", T::id()),
            column("experiment_id", T::id()).with_identity("instance"),
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

/// Quantities derived from the KKT-point analysis at a qualified candidate (ADR-0118
/// items 2, 3 and 10; Plan 22 S1). Every requested quantity has one `local_validity` row
/// stating whether it is certified or why it is withheld; a withheld quantity has no data
/// rows, so its reason has this home of its own (PS-12). The vocabularies of the fit
/// covariance, its intervals and their propagation are declared here once, with the
/// quantities that use them.
fn declare_local_analysis(b: &mut RegistryBuilder) {
    b.declare_enum(EnumDecl::platform(
        "DerivedQuantity",
        vec![
            EnumMember::new(
                "parametric_sensitivity",
                "Physical parameter derivatives of a regular square root solution, or the primal/dual solution and optimal value of an optimizing NLP.",
            ),
            EnumMember::new(
                "reduced_hessian",
                "The reduced Hessian over declared parameters: the second derivative of the optimal value.",
            ),
            EnumMember::new(
                "parameter_covariance",
                "The covariance of fitted parameters under the declared statistical model.",
            ),
            EnumMember::new(
                "wald_interval",
                "The Wald confidence intervals of the fitted parameters, from their covariance.",
            ),
            EnumMember::new(
                "profile_interval",
                "The profile-likelihood confidence intervals of the fitted parameters, from adaptive pin chains.",
            ),
            EnumMember::new(
                "propagated_covariance",
                "Parameter covariance propagated to outputs, Σ_y = J·Σ_θ·Jᵀ.",
            ),
        ],
    ));
    b.declare_enum(EnumDecl::platform(
        "WithheldReason",
        vec![
            EnumMember::new("not_feasible", "Original physical equality feasibility is insufficient for square response."),
            EnumMember::new("structural_unavailable", "Complete original equality support does not establish a square closure."),
            EnumMember::new("neighborhood_unavailable", "A state bound, guard or selector lacks an admitted local interior."),
            EnumMember::new(
                "no_candidate",
                "No candidate was observed in original coordinates.",
            ),
            EnumMember::new(
                "no_local_analysis",
                "The route ran no applicable local analysis at its candidate: a coefficient/cone route or a global incumbent adopted without fixed-assignment re-solve.",
            ),
            EnumMember::new(
                "multipliers_unrecovered",
                "A multiplier of some original row or bound is missing or failed recovery, postsolve included.",
            ),
            EnumMember::new(
                "complementarity_failed",
                "The recovered multipliers fail original-coordinate complementarity.",
            ),
            EnumMember::new(
                "not_stationary",
                "The candidate is not qualified stationary or better in original coordinates.",
            ),
            EnumMember::new(
                "analysis_unavailable",
                "The KKT-point analysis produced no point: no exact Hessian, the entry ceiling, or a failed evaluation or factorization.",
            ),
            EnumMember::new(
                "licq_failed",
                "The active constraint gradients are linearly dependent.",
            ),
            EnumMember::new(
                "weakly_active",
                "An active constraint's multiplier is within the dual budget of zero: strict complementarity fails.",
            ),
            EnumMember::new(
                "second_order_failed",
                "Second-order sufficiency does not hold at the candidate.",
            ),
            EnumMember::new(
                "backsolve_failed",
                "A backsolve or eigen-decomposition against the KKT factor failed.",
            ),
            EnumMember::new(
                "rank_deficient",
                "The scaled square state Jacobian or fitted parameter responses fail the stated numerical rank cutoff.",
            ),
            EnumMember::new(
                "nonunit_importance",
                "An included observation has an importance weight other than one.",
            ),
            EnumMember::new(
                "responses_unavailable",
                "The fit's local responses at the candidate are unavailable, so its response rank is unknown: a closure that is not square, the dense allowance, or a failed evaluation.",
            ),
            EnumMember::new(
                "parameter_at_bound",
                "A fitted parameter lies at a declared bound: the estimate is held there, and its local curvature does not describe its distribution.",
            ),
            EnumMember::new(
                "upstream_withheld",
                "A quantity this one is computed from was withheld.",
            ),
        ],
    ));
    b.declare_enum(EnumDecl::platform(
        "CovarianceApproximation",
        vec![
            EnumMember::new(
                "exact",
                "The inverse reduced Hessian of the fit's exact KKT analysis.",
            ),
            EnumMember::new(
                "gauss_newton",
                "The Gauss–Newton covariance from the response singular value decomposition; it neglects residual curvature.",
            ),
        ],
    ));
    b.declare_enum(EnumDecl::platform(
        "IntervalMethod",
        vec![
            EnumMember::new(
                "wald",
                "A Wald interval from the covariance and a quantile.",
            ),
            EnumMember::new(
                "profile_likelihood",
                "A profile-likelihood interval from adaptive pin chains.",
            ),
        ],
    ));
    b.declare_enum(EnumDecl::platform(
        "IntervalEnd",
        vec![
            EnumMember::new("lower", "The interval's lower end."),
            EnumMember::new("upper", "The interval's upper end."),
        ],
    ));
    b.declare_enum(EnumDecl::platform(
        "IntervalOutcome",
        vec![
            EnumMember::new(
                "threshold",
                "The end is where the interval's statistic reaches its quantile: a Wald end, or a profile point within the chain tolerance of the likelihood-ratio threshold.",
            ),
            EnumMember::new(
                "bound",
                "The profile reached the parameter's declared bound below the threshold: the end is the bound, and the interval is cut there by the admissible domain.",
            ),
            EnumMember::new(
                "stopped",
                "The profile chain stopped before the threshold or the bound: a pinned fit failed at the smallest step, the point budget or the deadline ran out, or a pinned fit found an objective below the estimate's. The end has no value; the detail states why.",
            ),
        ],
    ));
    let validity = record(vec![
        ("certified", flag()),
        ("reason", T::enumeration("WithheldReason").optional()),
        ("detail", text().optional()),
        ("conditional", flag()),
        ("licq", flag().optional()),
        ("strict_complementarity", flag().optional()),
        ("second_order", flag().optional()),
        ("weakly_active", ordinal().optional()),
        ("condition_1norm", real().optional()),
        ("residual", real().optional()),
        ("root_rank", ordinal().optional()),
        ("root_rank_cutoff", real().optional()),
        ("root_rank_relative_cutoff", real().optional()),
        ("root_backward_error", real().optional()),
        ("root_backward_error_limit", real().optional()),
        ("root_neighborhood", text().optional()),
    ])
    .named("LocalValidity");
    relation(
        b,
        N::Runtime,
        "local_validity",
        S::Derived,
        &["run_id", "step", "quantity"],
        vec![
            run_id(),
            column("step", ordinal()),
            column("quantity", T::enumeration("DerivedQuantity")),
            documented(
                "validity",
                validity,
                "Whether the quantity is certified or why it is withheld (`reason`, with the typed cause in `detail`), whether it is conditional on a discrete assignment, and the verdicts of the KKT point it was read from when the analysis ran: independent active gradients, strict complementarity, second-order sufficiency, the count of weakly active constraints, the 1-norm condition estimate of the normalized KKT matrix and the backward error of a refined backsolve.",
            ),
        ],
        "One row per quantity a step requested (PS-12, ADR-0118 item 10). A certified quantity's data rows are in its own relation; a withheld quantity has none, and this row states why.",
    );
    relation(
        b,
        N::Runtime,
        "parametric_sensitivities",
        S::Derived,
        &["run_id", "step", "parameter_id", "target_kind", "target_id"],
        vec![
            run_id(),
            column("step", ordinal()),
            column("parameter_id", T::id()),
            column("target_kind", T::enumeration("NumericalTarget")),
            documented(
                "target_id",
                T::id(),
                "The variable or row differentiated; nil for the objective.",
            ),
            column("parameter_unit_id", T::id()),
            documented(
                "target_unit_id",
                T::id(),
                "The unit of the variable, of the row's quantity, or of the objective's quantity.",
            ),
            documented(
                "primal",
                real(),
                "For a variable, dx/dp in its unit per parameter unit; for the objective, df*/dp in the authored sense; absent for a row.",
            )
            .optional(),
            documented(
                "dual",
                real(),
                "For a row, the derivative of its multiplier; for a variable, of its bound multiplier z_L - z_U; both in the minimization convention of `solve_constraints.dual`, zero where the constraint is inactive; absent for the objective.",
            )
            .optional(),
        ],
        "Local parametric sensitivities at a certified KKT point (ADR-0118 items 5–7): original physical units per parameter unit, in original coordinates whatever presolve removed. A first-order statement about the local solution map, valid while the active set holds; the validity is in local_validity.",
    );
    relation(
        b,
        N::Runtime,
        "reduced_hessians",
        S::Derived,
        &["run_id", "step"],
        vec![
            run_id(),
            column("step", ordinal()),
            documented(
                "parameters",
                T::list(T::id()),
                "The parameters, in request order: the order of every list in the row.",
            ),
            column("parameter_units", T::list(T::id())),
            column("objective_unit_id", T::id()),
            documented(
                "coordinate_scales",
                T::list(real()),
                "Each parameter's resolved coordinate scale S_p.",
            ),
            documented(
                "objective_scale",
                real(),
                "The objective's resolved coordinate scale S_f.",
            ),
            documented(
                "values",
                T::list(real()),
                "d²f*/dp² row-major, in the authored objective sense: objective units per row-parameter unit per column-parameter unit.",
            ),
            documented(
                "normalized",
                T::list(real()),
                "The dimensionless S_p·H·S_p / S_f, row-major.",
            ),
            documented(
                "eigenvalues",
                T::list(real()),
                "The eigenvalues of the normalized matrix, ascending.",
            ),
            documented(
                "eigenvectors",
                T::list(real()),
                "Its unit eigenvectors, column-major: column k belongs to eigenvalue k.",
            ),
        ],
        "The reduced Hessian over declared parameters at a certified KKT point (ADR-0118 item 6): the second derivative of the optimal value, read over the parameter pin rows. Its eigen-decomposition is taken in the declared coordinate scales, where it does not depend on the choice of units. The validity is in local_validity.",
    );
    declare_fit_uncertainty(b);
}

/// A fit's covariance, its intervals, the profile chains behind them and its response
/// directions (ADR-0118 items 1, 3 and 8; Plan 22 S3). The fit's validity rows are in
/// `local_validity` at step 0.
fn declare_fit_uncertainty(b: &mut RegistryBuilder) {
    relation(
        b,
        N::Runtime,
        "parameter_covariances",
        S::Derived,
        &["run_id"],
        vec![
            run_id(),
            documented(
                "approximation",
                T::enumeration("CovarianceApproximation"),
                "Exact, the inverse reduced Hessian of the fit's KKT analysis, when the fit used the exact Hessian; Gauss–Newton otherwise, which neglects residual curvature and is valid under the declared model with small residuals.",
            ),
            documented(
                "parameters",
                T::list(T::id()),
                "The free fitted parameters in fit order: the order of every list in the row.",
            ),
            column("parameter_units", T::list(T::id())),
            documented(
                "values",
                T::list(real()),
                "Σ row-major, in row-parameter unit × column-parameter unit.",
            ),
        ],
        "The covariance of a fit's free parameters at a certified estimate (ADR-0118 item 8), under the declared statistical model: weighted least squares with a declared standard deviation and unit importance for every included observation, the deviations taken as absolute, so no residual variance rescales it. One covariance per fit; the validity is in local_validity.",
    );
    relation(
        b,
        N::Runtime,
        "parameter_intervals",
        S::Derived,
        &["run_id", "parameter_id", "method", "end"],
        vec![
            run_id(),
            column("parameter_id", T::id()),
            column("method", T::enumeration("IntervalMethod")),
            column("end", T::enumeration("IntervalEnd")),
            column("unit_id", T::id()),
            documented("level", real(), "The confidence level in (0, 1)."),
            column("estimate", real()),
            documented(
                "value",
                real(),
                "The end in the parameter's unit; absent when the chain stopped.",
            )
            .optional(),
            column("outcome", T::enumeration("IntervalOutcome")),
            documented(
                "points",
                ordinal(),
                "Pinned fits the end's profile chain solved; zero for a Wald end.",
            ),
            documented(
                "detail",
                text(),
                "Why a chain stopped: the typed cause of its last failure.",
            )
            .optional(),
        ],
        "Confidence intervals of a fit's free parameters (ADR-0118 item 8). A Wald end is the estimate ± z·σ with z the standard normal quantile of (1 + level)/2. A profile-likelihood end is the pinned value at which the signed root of twice the objective increase, √(2(f - f*)), reaches √χ²₁(level), found by an adaptive pin chain; with absolute deviations both use the same quantile, so they agree on a linear model. The validity of each method's intervals is in local_validity.",
    );
    relation(
        b,
        N::Runtime,
        "profile_points",
        S::Derived,
        &["run_id", "parameter_id", "end", "point"],
        vec![
            run_id(),
            column("parameter_id", T::id()),
            column("end", T::enumeration("IntervalEnd")),
            documented(
                "point",
                ordinal(),
                "The point's position in its chain, in solve order.",
            ),
            documented(
                "value",
                real(),
                "The pinned parameter value, in the parameter's unit.",
            ),
            documented(
                "seed",
                ordinal(),
                "The point whose solution seeded this pinned fit; absent when the fit estimate seeded it (PS-11).",
            )
            .optional(),
            column("qualification", T::enumeration("NativeQualification")).optional(),
            documented(
                "objective",
                real(),
                "The pinned fit's objective f, half the weighted residual sum of squares.",
            )
            .optional(),
            documented(
                "statistic",
                real(),
                "√(2·max(f - f*, 0)) against the estimate's objective f*.",
            )
            .optional(),
            documented(
                "accepted",
                flag(),
                "The pinned fit was qualified stationary or better and feasible, and the chain used it.",
            ),
            documented(
                "detail",
                text(),
                "The typed cause of a failed pinned fit.",
            )
            .optional(),
        ],
        "Every pinned fit of a profile-likelihood chain (ADR-0118 item 8): one chain per free parameter and end, each point pinning the parameter on the fit prepared once and seeded from its chain's latest accepted point, which is recorded as its input.",
    );
    relation(
        b,
        N::Runtime,
        "response_directions",
        S::Derived,
        &["run_id", "direction", "parameter_id"],
        vec![
            run_id(),
            documented(
                "direction",
                ordinal(),
                "The right singular vector's position, by decreasing singular value.",
            ),
            column("parameter_id", T::id()),
            documented(
                "singular_value",
                real(),
                "Its singular value; zero beyond the number of included observations.",
            ),
            documented(
                "identifiable",
                flag(),
                "The singular value exceeds the fit's relative rank cutoff.",
            ),
            documented(
                "component",
                real(),
                "The direction's component along the parameter, in coordinates divided by the parameter's declared scale.",
            ),
        ],
        "The right singular vectors of a fit's weighted, parameter-scaled response matrix at its candidate (ADR-0118 item 8): the identifiable directions span the locally identifiable subspace, and the others are the parameter combinations the observations do not determine.",
    );
    relation(
        b,
        N::Runtime,
        "propagated_covariances",
        S::Derived,
        &["run_id", "step"],
        vec![
            run_id(),
            column("step", ordinal()),
            documented(
                "covariance_run_id",
                T::id(),
                "The fit run whose parameter covariance was propagated: this run for a fit's own predictions.",
            )
            .with_identity("run"),
            documented(
                "parameters",
                T::list(T::id()),
                "The fitted parameters propagated, in the covariance's order.",
            ),
            documented(
                "outputs",
                T::list(T::id()),
                "The solved variables of a modeling step, or the included observations whose predictions a fit propagates: the order of every other output list.",
            ),
            column("output_units", T::list(T::id())),
            documented(
                "values",
                T::list(real()),
                "Σ_y row-major, in row-output unit × column-output unit.",
            ),
        ],
        "A parameter covariance propagated to outputs, Σ_y = J·Σ_θ·Jᵀ (ADR-0118 items 1 and 11; the counterpart of IDAES sens.py): J is a modeling step's parametric sensitivities over the covariance's parameters, matched by identity, or a fit's response derivatives. A first-order statement valid while both its inputs are; its validity, the conjunction of theirs, is in local_validity.",
    );
}
