// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared 6.13 execution and evidence contracts.
use super::declarations::{column, relation, structure};
use crate::RegistryBuilder;
use crate::model::{FieldContract, FieldContract as T, Namespace as N, SnapshotClass as S};
/// Declare the structural contracts; no later pass is implemented by these declarations.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_provenance(builder);
    declare_diagnostics(builder);
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

/// The sole diagnostic-field declaration. Only an execution finding without a check
/// origin may omit `check_id` inside an attributed terminal pass record.
pub fn diagnostic_columns(execution_finding: bool) -> Vec<FieldContract> {
    let mut fields = vec![
        column("finding_id", T::id()),
        column("run_id", T::id()).optional(),
        column("check_id", T::id()),
        column("severity", T::enumeration("FindingSeverity")),
        column("subjects", T::list(T::id())),
        column("evidence", diagnostic_evidence()),
        column("message", T::native(arrow_schema::DataType::Utf8)),
        column(
            "next_steps",
            T::list(T::native(arrow_schema::DataType::Utf8)),
        ),
    ];
    fields[2] = fields[2].clone().with_nullable(execution_finding);
    fields
}

fn diagnostic_evidence() -> T {
    T::structure(vec![
        T::native(arrow_schema::DataType::Utf8).with_name("kind"),
        T::structure(vec![
            T::id().with_name("relation_id"),
            T::row_key().with_name("row_key"),
        ])
        .with_name("row")
        .optional(),
        T::structure(vec![
            T::enumeration("FailureClass").with_name("failure_class"),
            T::native(arrow_schema::DataType::Utf8)
                .with_name("diagnostic_code")
                .optional(),
            T::native(arrow_schema::DataType::Utf8).with_name("attempt_error"),
        ])
        .with_name("execution")
        .optional(),
    ])
    .with_alternative(&crate::model::TaggedAlternative::new(
        "kind",
        [
            ("row".into(), "row".into()),
            ("execution".into(), "execution".into()),
        ],
    ))
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
            column("row_key", T::row_key()),
            column("rule_id", T::id()).optional(),
            column("algorithm_id", T::id()).optional(),
            column(
                "supporting",
                T::list(structure(vec![
                    ("relation_id", T::id()),
                    ("row_key", T::row_key()),
                ])),
            ),
        ],
        "blueprint §6.13 derivation evidence.",
    );
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
