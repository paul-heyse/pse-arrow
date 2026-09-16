// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared 6.13 execution and evidence contracts.
use super::declarations::{column, relation, relation_version, structure};
use crate::RegistryBuilder;
use crate::model::{FieldContract, FieldContract as T, Namespace as N, SnapshotClass as S};
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
    declare_diagnostics(builder);
    declare_runs(builder);
    declare_solutions(builder);
    declare_duals(builder);
    declare_residuals(builder);
    declare_iterations(builder);
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

/// Minimal shared diagnostic foundation for an otherwise explicit custom registry.
pub(super) fn declare_diagnostics(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "FindingSeverity",
        crate::model::Severity::ALL
            .iter()
            .map(|value| value.as_str()),
    );
    declare_diagnostics_findings(builder);
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
            column("attempt", T::nonnegative(i64::from(u16::MAX))),
            column(
                "resolved_options",
                T::list(structure(vec![
                    ("key", T::native(arrow_schema::DataType::Utf8)),
                    ("value", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column(
                "started_at",
                T::native(crate::model::extension::timestamp_storage()),
            ),
            column(
                "finished_at",
                T::native(crate::model::extension::timestamp_storage()),
            ),
            column("status", T::enumeration("TerminationStatus")),
            column(
                "environment",
                structure(vec![
                    ("platform_version", T::native(arrow_schema::DataType::Utf8)),
                    ("compiler_version", T::native(arrow_schema::DataType::Utf8)),
                    ("solver_version", T::native(arrow_schema::DataType::Utf8)),
                    ("kernel_digests", T::list(T::hash())),
                    ("host", T::native(arrow_schema::DataType::Utf8)),
                ]),
            ),
            column("wall_seconds", T::native(arrow_schema::DataType::Float64)),
            column("iterations", T::nonnegative(i64::from(u32::MAX))).optional(),
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
            column("value", T::native(arrow_schema::DataType::Float64)),
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
            column("dual", T::native(arrow_schema::DataType::Float64)),
            column(
                "bound_multiplier_lower",
                T::native(arrow_schema::DataType::Float64),
            )
            .optional(),
            column(
                "bound_multiplier_upper",
                T::native(arrow_schema::DataType::Float64),
            )
            .optional(),
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
            column("residual", T::native(arrow_schema::DataType::Float64)),
            column(
                "scaled_residual",
                T::native(arrow_schema::DataType::Float64),
            ),
            column(
                "relative_residual",
                T::native(arrow_schema::DataType::Float64),
            )
            .optional(),
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
            column("iteration", T::nonnegative(i64::from(u32::MAX))),
            column("objective", T::native(arrow_schema::DataType::Float64)),
            column("inf_pr", T::native(arrow_schema::DataType::Float64)),
            column("inf_du", T::native(arrow_schema::DataType::Float64)),
            column("mu", T::native(arrow_schema::DataType::Float64)),
            column("step_size", T::native(arrow_schema::DataType::Float64)),
            column("regularization", T::native(arrow_schema::DataType::Float64)),
            column("restoration", T::native(arrow_schema::DataType::Boolean)),
        ],
        "blueprint §6.13 execution and evidence: iterations.",
    );
}

/// The sole diagnostic-field declaration. Only an execution finding without a check
/// origin may omit `check_id` inside an attributed terminal pass record.
pub fn diagnostic_columns(execution_finding: bool) -> Vec<FieldContract> {
    let mut fields = vec![
        column("finding_id", T::id()),
        column("subject_snapshot", T::hash()).optional(),
        column("run_id", T::id()).optional(),
        column("check_id", T::id()),
        column("severity", T::enumeration("FindingSeverity")),
        column("subjects", T::list(T::id())),
        column("values", T::native(arrow_schema::DataType::Utf8)),
        column("message", T::native(arrow_schema::DataType::Utf8)),
        column(
            "next_steps",
            T::list(T::native(arrow_schema::DataType::Utf8)),
        ),
    ];
    fields[3] = fields[3].clone().with_nullable(execution_finding);
    fields
}

fn diagnostic_type() -> T {
    T::structure(diagnostic_columns(true))
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
            column("row_ordinal", T::nonnegative(i64::MAX)),
            column("output_ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("result", kernel_result()),
        ],
        "blueprint §6.13 execution and evidence: kernel_evaluation_outcomes.",
    );
}

fn kernel_result() -> T {
    let alternative = crate::model::TaggedAlternative::new(
        "kind",
        [
            ("success".into(), "success".into()),
            ("missing_input".into(), "failure".into()),
            ("domain_failure".into(), "failure".into()),
            ("implementation_failure".into(), "failure".into()),
        ],
    );
    T::structure(vec![
        T::enumeration("KernelOutcome").with_name("kind"),
        T::extended(crate::model::ExtensionUse::QuantityValue)
            .with_name("success")
            .optional(),
        T::structure(vec![
            T::enumeration("KernelFailure").with_name("reason_code"),
            T::id().with_name("quantity_type_id"),
            T::id().with_name("unit_id"),
        ])
        .with_name("failure")
        .optional(),
    ])
    .with_alternative(&alternative)
}

fn declare_host_capabilities(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "host_capabilities",
        S::Derived,
        &["host"],
        vec![
            column("host", T::native(arrow_schema::DataType::Utf8)),
            column(
                "probed_at",
                T::native(crate::model::extension::timestamp_storage()),
            ),
            column("ipopt_version", T::native(arrow_schema::DataType::Utf8)),
            column(
                "linear_solvers",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column("hsl_available", T::native(arrow_schema::DataType::Boolean)),
            column("petsc_version", T::native(arrow_schema::DataType::Utf8)).optional(),
            column(
                "python_env",
                T::structure(vec![
                    T::native(arrow_schema::DataType::Utf8)
                        .with_name("interpreter")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::Utf8)
                        .with_name("pyomo")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::Utf8)
                        .with_name("pint")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::Utf8)
                        .with_name("pyarrow")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::Utf8)
                        .with_name("numpy")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::Utf8)
                        .with_name("scipy")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::Utf8)
                        .with_name("idaes")
                        .with_nullable(true),
                    T::list(structure(vec![
                        ("name", T::native(arrow_schema::DataType::Utf8)),
                        ("version", T::native(arrow_schema::DataType::Utf8)),
                    ]))
                    .with_name("pyomo_contrib")
                    .with_nullable(false),
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
            column("row_key", T::native(arrow_schema::DataType::Utf8)),
            column("rule_id", T::id()).optional(),
            column("pass_id", T::id()).optional(),
            column(
                "supporting",
                T::list(structure(vec![
                    ("relation_id", T::id()),
                    ("row_key", T::native(arrow_schema::DataType::Utf8)),
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
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("snapshot_id", T::hash()),
            column("manifest_checksum", T::hash()),
            column(
                "updated_at",
                T::native(crate::model::extension::timestamp_storage()),
            ),
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
            FieldContract::key("assertion_id", T::id(), "The authored assertion identity."),
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("expected", T::native(arrow_schema::DataType::Utf8)),
            column("status", T::enumeration("AssertionStatus")),
            column("reason", T::native(arrow_schema::DataType::Utf8)),
        ]),
    );
}
fn declare_pass_records(builder: &mut RegistryBuilder) {
    let derivations = builder
        .declared_relations()
        .iter()
        .find(|relation| relation.key.qualified_name() == "provenance.derivations")
        .map_or_else(
            || T::structure(Vec::new()),
            |relation| T::structure(relation.columns.clone()),
        );
    relation_version(
        builder,
        N::Provenance,
        "pass_records",
        2,
        S::Sidecar,
        &["pass_run_id"],
        vec![
            column("pass_run_id", T::id()),
            column("pass_id", T::id()),
            column("version", T::native(arrow_schema::DataType::Utf8)),
            column("snapshot_in", T::hash()).optional(),
            column("snapshot_out", T::hash()).optional(),
            column("engine_profile_hash", T::hash()).optional(),
            column(
                "plan_evidence",
                T::list(structure(vec![
                    ("encoding_checksum", T::hash()),
                    ("encoding", T::native(arrow_schema::DataType::Utf8)),
                    ("codec_version", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column(
                "plan_explain",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column(
                "rules_fired",
                T::list(structure(vec![
                    ("plan_ordinal", T::nonnegative(i64::from(u16::MAX))),
                    ("rule_name", T::native(arrow_schema::DataType::Utf8)),
                    ("ordinal", T::nonnegative(i64::from(u16::MAX))),
                ])),
            ),
            column("duration_ms", T::native(arrow_schema::DataType::Float64)),
            column("finding_count", T::nonnegative(i64::MAX)),
            column("status", T::enumeration("PassStatus")),
            column("findings", T::list(diagnostic_type())),
            column("failure_class", T::enumeration("FailureClass")).optional(),
            column("derivations", T::list(derivations)),
        ],
        "blueprint §6.13 noncanonical execution evidence.",
    );
}
