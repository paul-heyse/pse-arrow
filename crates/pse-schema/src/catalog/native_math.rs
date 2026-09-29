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
        2,
        S::Derived,
        &["backend"],
        vec![
            column("backend", T::enumeration("NativeBackend")),
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
        ],
        "Linked adapter inventory. Contextual eligibility is evaluated separately for the selected request. `automatic_classes` are the classes automatic routing may choose the adapter for; every other class in `classes` needs explicit selection. Automatic routing takes the problem's classes most specific first and selects among the eligible adapters automatic for the first class that has one. `certifies` marks an adapter that serves the explicit certify intent with global_bound and proven_infeasible assurances. `native_forms` lists the constraint handlers the adapter consumes; a structure that leaves any other form to a native handler is ineligible (ADR-0104).",
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
        2,
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
            "ipopt", "pounce", "kinsol", "highs", "clarabel", "diffsol", "idas", "scip",
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
            "first_with_smooth_sensitivities",
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
        4,
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
        ],
        "Actual native termination, independent original-model validation and explicit unattempted/error states. No candidate implies no claimed solution.",
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
        "A native infeasibility (Farkas) or unboundedness ray, in original physical coordinates over the cone form of the solved problem: rows then finite variable bounds, lower before upper. Verification recomputes it against the original data: `residual` is the worst column's |Aᵀy| (or row's |Px|, |Ax| on zero rows) relative to the magnitudes it sums, `objective` is bᵀy (or qᵀx) and `cone` the dual-cone (or cone) violation, both relative to the ray's largest entry; `margin` is -bᵀy less the rows' and bounds' acceptance budgets weighted by |y| (or -qᵀx), relative to the same entry. A ray is verified when every relative quantity is within `tolerance` and the margin is positive, so a problem infeasible by less than its acceptance budgets is never certified. Only a verified ray at full accuracy carries the certificate assurance. Absent verification means the original data could not be evaluated.",
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
        3,
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
                    ("observation_id", T::id()),
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
        "Shared-parameter fitting over authored modeling cases. Each experiment binds shared parameter identities to local source paths in canonical physical units. Observation paths select original members. Elapsed time is relative to the integration start; model_clock is the authored axis coordinate. Measurement values, units and uncertainty remain authored.observations.",
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
