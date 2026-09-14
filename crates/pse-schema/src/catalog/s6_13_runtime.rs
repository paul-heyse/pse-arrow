// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared 6.13 execution and evidence contracts.
use super::declarations::{column, relation, structure};
use crate::RegistryBuilder;
use crate::model::{ColumnSpec, LogicalType as T, Namespace as N, SnapshotClass as S};
/// Declare the structural contracts; no later pass is implemented by these declarations.
pub fn declare(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "PassStatus",
        crate::model::PassStatus::ALL
            .iter()
            .map(|value| value.as_str()),
    );
    declare_provenance(builder);
    super::terminal_attempts::declare(builder);
    super::declarations::enumeration(
        builder,
        "FindingSeverity",
        crate::model::Severity::ALL
            .iter()
            .map(|value| value.as_str()),
    );
    declare_runs(builder);
    declare_solutions(builder);
    declare_duals(builder);
    declare_residuals(builder);
    declare_iterations(builder);
    declare_diagnostics_findings(builder);
    declare_kernel_evaluations(builder);
    declare_kernel_evaluation_outcomes(builder);
    declare_host_capabilities(builder);
    super::declarations::enumeration(
        builder,
        "BoundStatus",
        ["interior", "at_lower", "at_upper", "violated"],
    );
    super::declarations::enumeration(
        builder,
        "KernelOutcome",
        [
            "success",
            "missing_input",
            "domain_failure",
            "implementation_failure",
        ],
    );
    super::declarations::enumeration(
        builder,
        "TerminationStatus",
        [
            "optimal",
            "locally_infeasible",
            "infeasible",
            "unbounded",
            "max_iterations",
            "max_time",
            "restoration_failed",
            "solver_error",
            "cancelled",
            "user_interrupt",
        ],
    );
    super::declarations::enumeration(builder, "AssertionStatus", ["pass", "fail", "obsolete"]);
}
fn declare_runs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "runs",
        S::Derived,
        &["run_id"],
        vec![
            column("run_id", T::id()),
            column("problem_id", T::id()),
            column("case_id", T::id()),
            column("model_revision_id", T::id()),
            column("backend", T::enumeration("Backend")),
            column("solver_profile_id", T::id()),
            column("plan_id", T::id()).optional(),
            column("stage_id", T::id()).optional(),
            column("parent_run_id", T::id()).optional(),
            column("attempt", T::U16),
            column(
                "resolved_options",
                T::list(structure(vec![("key", T::Text), ("value", T::Text)])),
            ),
            column("started_at", T::Timestamp),
            column("finished_at", T::Timestamp),
            column("status", T::enumeration("TerminationStatus")),
            column(
                "environment",
                structure(vec![
                    ("platform_version", T::Text),
                    ("compiler_version", T::Text),
                    ("solver_version", T::Text),
                    ("kernel_digests", T::list(T::hash())),
                    ("host", T::Text),
                ]),
            ),
            column("wall_seconds", T::F64),
            column("iterations", T::U32).optional(),
        ],
        "blueprint §6.13 execution and evidence: runs.",
    );
}

fn declare_solutions(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "solutions",
        S::Derived,
        &["run_id", "symbol_id"],
        vec![
            column("run_id", T::id()),
            column("symbol_id", T::id()),
            column("value", T::F64),
            column("unit_id", T::id()),
            column("bound_status", T::enumeration("BoundStatus")),
        ],
        "blueprint §6.13 execution and evidence: solutions.",
    );
}

fn declare_duals(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "duals",
        S::Derived,
        &["run_id", "equation_id"],
        vec![
            column("run_id", T::id()),
            column("equation_id", T::id()),
            column("dual", T::F64),
            column("bound_multiplier_lower", T::F64).optional(),
            column("bound_multiplier_upper", T::F64).optional(),
        ],
        "blueprint §6.13 execution and evidence: duals.",
    );
}

fn declare_residuals(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "residuals",
        S::Derived,
        &["run_id", "equation_id"],
        vec![
            column("run_id", T::id()),
            column("equation_id", T::id()),
            column("residual", T::F64),
            column("scaled_residual", T::F64),
            column("relative_residual", T::F64).optional(),
        ],
        "blueprint §6.13 execution and evidence: residuals.",
    );
}

fn declare_iterations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "iterations",
        S::Derived,
        &["run_id", "iteration"],
        vec![
            column("run_id", T::id()),
            column("iteration", T::U32),
            column("objective", T::F64),
            column("inf_pr", T::F64),
            column("inf_du", T::F64),
            column("mu", T::F64),
            column("step_size", T::F64),
            column("regularization", T::F64),
            column("restoration", T::Bool),
        ],
        "blueprint §6.13 execution and evidence: iterations.",
    );
}

/// The sole diagnostic-field declaration. Only an execution finding without a check
/// origin may omit `check_id` inside an attributed terminal pass record.
pub fn diagnostic_columns(execution_finding: bool) -> Vec<ColumnSpec> {
    let mut fields = vec![
        column("finding_id", T::id()),
        column("subject_snapshot", T::hash()).optional(),
        column("run_id", T::id()).optional(),
        column("check_id", T::id()),
        column("severity", T::enumeration("FindingSeverity")),
        column("subjects", T::list(T::id())),
        column("values", T::Text),
        column("message", T::Text),
        column("next_steps", T::list(T::Text)),
    ];
    fields[3].nullable = execution_finding;
    fields
}

fn diagnostic_type() -> T {
    T::Struct(
        diagnostic_columns(true)
            .into_iter()
            .map(|column| (column.name, column.logical_type, column.nullable))
            .collect(),
    )
}

fn declare_diagnostics_findings(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "diagnostics_findings",
        S::Derived,
        &["finding_id"],
        diagnostic_columns(false),
        "blueprint §6.13 execution and evidence: diagnostics_findings.",
    );
}

fn declare_kernel_evaluations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "kernel_evaluations",
        S::Derived,
        &["evaluation_id"],
        vec![
            column("evaluation_id", T::id()),
            column("kernel_binding_id", T::id()),
            column("input_hash", T::hash()),
            column("output_batch_hash", T::hash()),
        ],
        "blueprint §6.13 execution and evidence: kernel_evaluations.",
    );
}

fn declare_kernel_evaluation_outcomes(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "kernel_evaluation_outcomes",
        S::Derived,
        &["evaluation_id", "row_ordinal", "output_ordinal"],
        vec![
            column("evaluation_id", T::id()),
            column("row_ordinal", T::U64),
            column("output_ordinal", T::U16),
            column("outcome", T::enumeration("KernelOutcome")),
            column("value", T::F64).optional(),
            column("quantity_type_id", T::id()),
            column("unit_id", T::id()),
            column("reason_code", T::enumeration("KernelFailure")).optional(),
        ],
        "blueprint §6.13 execution and evidence: kernel_evaluation_outcomes.",
    );
}

fn declare_host_capabilities(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "host_capabilities",
        S::Derived,
        &["host"],
        vec![
            column("host", T::Text),
            column("probed_at", T::Timestamp),
            column("ipopt_version", T::Text),
            column("linear_solvers", T::list(T::Text)),
            column("hsl_available", T::Bool),
            column("petsc_version", T::Text).optional(),
            column(
                "python_env",
                T::Struct(vec![
                    ("interpreter", T::Text, false),
                    ("pyomo", T::Text, false),
                    ("pint", T::Text, false),
                    ("pyarrow", T::Text, false),
                    ("numpy", T::Text, false),
                    ("scipy", T::Text, false),
                    ("idaes", T::Text, true),
                    (
                        "pyomo_contrib",
                        T::list(structure(vec![("name", T::Text), ("version", T::Text)])),
                        false,
                    ),
                ]),
            )
            .optional(),
        ],
        "blueprint §6.13 execution and evidence: host_capabilities.",
    );
}

fn declare_provenance(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Provenance,
        "derivations",
        S::Sidecar,
        &["derivation_id"],
        vec![
            column("derivation_id", T::id()),
            column("relation_id", T::id()),
            column("row_key", T::Text),
            column("rule_id", T::id()).optional(),
            column("pass_id", T::id()).optional(),
            column(
                "supporting",
                T::list(structure(vec![
                    ("relation_id", T::id()),
                    ("row_key", T::Text),
                ])),
            ),
            column("snapshot_id", T::hash()).optional(),
            column("fingerprint", T::hash()).optional(),
        ],
        "blueprint §6.13 derivation evidence.",
    );
    relation(
        builder,
        N::Provenance,
        "refs",
        S::Sidecar,
        &["name"],
        vec![
            column("name", T::Text),
            column("snapshot_id", T::hash()),
            column("manifest_checksum", T::hash()),
            column("updated_at", T::Timestamp),
        ],
        "blueprint §6.13 mutable refs.",
    );
    declare_pass_records(builder);
    builder.declare_relation(
        crate::model::RelationDecl::new(
            N::Provenance,
            "assertions",
            1,
            crate::model::Authority::Authored,
            S::Sidecar,
            "blueprint §6.13 authored expected evidence; excluded from semantic membership.",
        )
        .pk(&["assertion_id"])
        .columns(vec![
            ColumnSpec::key("assertion_id", T::id(), "The authored assertion identity."),
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("expected", T::Text),
            column("status", T::enumeration("AssertionStatus")),
            column("reason", T::Text),
        ]),
    );
}
fn declare_pass_records(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Provenance,
        "pass_records",
        S::Sidecar,
        &["pass_run_id"],
        vec![
            column("pass_run_id", T::id()),
            column("pass_id", T::id()),
            column("version", T::Text),
            column("snapshot_in", T::hash()).optional(),
            column("snapshot_out", T::hash()).optional(),
            column("engine_profile_hash", T::hash()).optional(),
            column(
                "plan_evidence",
                T::list(structure(vec![
                    ("encoding_checksum", T::hash()),
                    ("encoding", T::Text),
                    ("codec_version", T::Text),
                ])),
            ),
            column("plan_explain", T::list(T::Text)),
            column(
                "rules_fired",
                T::list(structure(vec![
                    ("plan_ordinal", T::U16),
                    ("rule_name", T::Text),
                    ("ordinal", T::U16),
                ])),
            ),
            column("duration_ms", T::F64),
            column("finding_count", T::U64),
            column("status", T::enumeration("PassStatus")),
            column("findings", T::list(diagnostic_type())),
            column("failure_class", T::enumeration("FailureClass")).optional(),
        ],
        "blueprint §6.13 noncanonical execution evidence.",
    );
}
