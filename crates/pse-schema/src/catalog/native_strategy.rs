// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Numerical composition vocabulary and actual, ordered execution observations.
use super::declarations::{column, documented, enumeration, relation, run_id};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;

pub(super) fn declare(b: &mut RegistryBuilder) {
    enumeration(b, "NumericalCompositionPolicy", ["auto", "declared"]);
    enumeration(
        b,
        "NumericalMechanism",
        [
            "direct",
            "kkt_predictor",
            "activity_path",
            "limited_corrector",
            "root_response",
            "secant_predictor",
            "newton_krylov",
            "setup_reuse",
            "maps_anderson",
            "bounded_feasibility",
            "native_globalization",
            "continuation",
            "homotopy",
            "pseudo_transient",
            "consistent_initialization",
            "block",
            "reduced_space",
            "nonlinear_preconditioner",
            "surrogate",
            "multifidelity",
            "schur",
            "batch",
            "multistart",
            "selected_evaluation",
            "least_deviation",
        ],
    );
    enumeration(
        b,
        "NumericalPosition",
        ["preparation", "execution", "recovery"],
    );
    enumeration(
        b,
        "NumericalPhase",
        [
            "preparation",
            "prediction",
            "screening",
            "native",
            "verification",
            "assessment",
        ],
    );
    enumeration(
        b,
        "NumericalScope",
        ["task", "occurrence", "mechanism", "attempt"],
    );
    enumeration(
        b,
        "NumericalAccuracyClass",
        ["certified", "estimated", "unresolved"],
    );
    enumeration(
        b,
        "NumericalStartOrigin",
        [
            "specification",
            "explicit",
            "accepted",
            "predicted",
            "auxiliary",
            "partial",
            "modified_specification",
            "surrogate",
        ],
    );
    enumeration(
        b,
        "NumericalTransition",
        ["finish", "continue", "subdivide", "recover", "stop"],
    );
    enumeration(b, "NumericalBranchPolicy", ["any_qualified", "connected"]);
    enumeration(
        b,
        "NumericalEventKind",
        ["planned", "started", "finished", "refused", "abandoned"],
    );
    enumeration(
        b,
        "NumericalPathEventKind",
        ["regular", "simple_fold", "unresolved"],
    );
    enumeration(
        b,
        "NumericalAttemptObservation",
        [
            "converged",
            "stalled",
            "limited",
            "numerical_failure",
            "capability_refusal",
            "contract_failure",
            "resource_exhausted",
            "cancelled",
            "panic",
            "operational_failure",
            "auxiliary",
            "infeasible",
        ],
    );
    enumeration(
        b,
        "NumericalOriginalConclusion",
        ["satisfied", "refused", "unavailable"],
    );
    relation(
        b, N::Runtime, "solve_strategy_events", S::Derived,
        &["run_id", "step", "event"],
        vec![
            run_id(),
            column("step", T::nonnegative(i64::from(u32::MAX))),
            column("event", T::nonnegative(i64::from(u32::MAX))),
            column("strategy_identity", T::hash()),
            documented("decision_identity", T::hash(), "Identity of the actual bound automatic or declared execution decision.").optional(),
            column("mechanism", T::enumeration("NumericalMechanism")),
            column("kind", T::enumeration("NumericalEventKind")),
            column("phase", T::enumeration("NumericalPhase")),
            column("scope", T::enumeration("NumericalScope")),
            column("charging_owner", T::hash()).optional(),
            column("original_identity", T::hash()),
            documented("original_conclusion", T::enumeration("NumericalOriginalConclusion"), "Independent original-model assessment; native termination remains separate.").optional(),
            column("derived_identity", T::hash()).optional(),
            column("profile_identity", T::hash()).optional(),
            column("backend", T::enumeration("NativeBackend")).optional(),
            column("start_origin", T::enumeration("NumericalStartOrigin")).optional(),
            column("start_identity", T::hash()).optional(),
            column("transport_identity", T::hash()).optional(),
            column("accuracy_class", T::enumeration("NumericalAccuracyClass")).optional(),
            column("accuracy_identity", T::hash()).optional(),
            documented("fidelity_identity", T::hash(), "Actual surrogate evaluation fidelity; absent for nonstatistical operations.").optional(),
            documented("surrogate_task_identity", T::hash(), "Complete retained library model-management options and correspondence key.").optional(),
            documented("infill_statistic", T::native(D::Float64), "Observed statistical infill statistic; never a deterministic numerical accuracy certificate.").optional(),
            column("surrogate_radius", T::native(D::Float64)).optional(),
            column("surrogate_global_iterations", T::nonnegative(i64::MAX)).optional(),
            column("surrogate_local_iterations", T::nonnegative(i64::MAX)).optional(),
            documented("surrogate_coordinates", T::list(T::native(D::Float64)), "Library proposal in complete physical original-coordinate order; requires independent original screening/correction.").optional(),
            documented("surrogate_model_values", T::list(T::native(D::Float64)), "Actual approximation outputs; do not grant original feasibility or result permission.").optional(),
            documented("path_events", T::list(T::structure(vec![
                column("kind", T::enumeration("NumericalPathEventKind")),
                column("source", T::hash()),
                column("family", T::hash()),
                column("point", T::list(T::native(D::Float64))),
                column("localization", T::structure(vec![
                    column("left", T::hash()), column("right", T::hash()),
                    column("lower", T::native(D::Float64)), column("upper", T::native(D::Float64)),
                    column("at", T::native(D::Float64)),
                ])).optional(),
                column("state_rank", T::nonnegative(i64::MAX)),
                column("augmented_rank", T::nonnegative(i64::MAX)),
                column("state_singular_values", T::list(T::native(D::Float64))),
                column("augmented_singular_values", T::list(T::native(D::Float64))),
                column("rank_threshold", T::native(D::Float64)),
                column("transversality", T::native(D::Float64)).optional(),
                column("curvature", T::native(D::Float64)).optional(),
                column("curvature_source", T::hash()).optional(),
                column("nondegeneracy_threshold", T::native(D::Float64)),
                column("values", T::nonnegative(i64::MAX)),
                column("state_actions", T::nonnegative(i64::MAX)),
                column("parameter_actions", T::nonnegative(i64::MAX)),
                column("curvature_actions", T::nonnegative(i64::MAX)),
                column("factorizations", T::nonnegative(i64::MAX)),
                column("backsolves", T::nonnegative(i64::MAX)),
                column("rank_probes", T::nonnegative(i64::MAX)),
            ])), "Actual localized numerical path observations and attempted work; rank/nondegeneracy interpretations are estimated, never formal certificates. Missing curvature remains absent.").optional(),
            column("observation", T::enumeration("NumericalAttemptObservation")).optional(),
            column("transition", T::enumeration("NumericalTransition")).optional(),
            column("permission", T::enumeration("CandidateUse")).optional(),
            documented("attempts", T::nonnegative(i64::MAX), "Actual started executions attributed once to the charging owner.").optional(),
            documented("evaluations", T::nonnegative(i64::MAX), "Actual evaluations including rejected trials; absent when not observable.").optional(),
            column("iterations", T::nonnegative(i64::MAX)).optional(),
            column("factorizations", T::nonnegative(i64::MAX)).optional(),
            column("proof_steps", T::nonnegative(i64::MAX)).optional(),
            column("failures", T::list(T::structure(super::modeling_analysis::diagnostic_fields_with_causes()))),
            documented("detail", T::native(D::Utf8), "Presentation only; transitions and permissions derive from typed observations.").optional(),
        ],
        "Ordered shared numerical driver events, including preparation and report-less failures. Plans are distinguished from executions; native exit observations do not grant original candidate permission. Unknown work remains absent and inclusive work has one charging owner. Numerical attempts stay within the existing durable occurrence.",
    );
}
