// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete pass-attempt admission as declared relational invariants (blueprint §14.3).
use super::inv::{declare as invariant, filter, project, scan};
use crate::{
    RegistryBuilder,
    model::{Cell, CmpOp, EmptyListPolicy, InvariantKind, NullListPolicy, RuleExpr as E, RulePlan},
};

const RELATION: &str = "provenance.pass_records";
fn eq(name: &'static str, value: &'static str) -> E {
    E::cmp(CmpOp::Eq, E::Col(name), E::Lit(Cell::Enum(value)))
}
fn null(name: &'static str) -> E {
    E::IsNull(Box::new(E::Col(name)))
}
fn failed() -> E {
    E::Or(vec![eq("status", "failed"), eq("status", "cancelled")])
}
fn check(builder: &mut RegistryBuilder, name: &str, predicate: E, doc: &'static str) {
    invariant(
        builder,
        RELATION,
        name,
        InvariantKind::Check,
        &["pass_run_id"],
        project(
            filter(scan(RELATION, "subject"), predicate),
            &["pass_run_id"],
        ),
        doc,
    );
}
pub(super) fn declare(builder: &mut RegistryBuilder) {
    check(
        builder,
        "check:terminal_count",
        E::cmp(
            CmpOp::NotEq,
            E::Col("finding_count"),
            E::ListLen(Box::new(E::Col("findings"))),
        ),
        "The recorded finding count equals the actual complete typed finding list length.",
    );
    check(
        builder,
        "check:terminal_failure_class",
        E::IsNotDistinctFrom(Box::new(failed()), Box::new(null("failure_class"))),
        "Failure class is present exactly for failed or cancelled attempts.",
    );
    check(
        builder,
        "check:terminal_cancel_class",
        E::IsDistinctFrom(
            Box::new(eq("status", "cancelled")),
            Box::new(E::IsTrue(Box::new(eq(
                "failure_class",
                "runtime.cancelled",
            )))),
        ),
        "Cancelled attempts have exactly the runtime.cancelled failure class.",
    );
    check(
        builder,
        "check:terminal_failed_output",
        E::And(vec![
            failed(),
            E::IsNotNull(Box::new(E::Col("snapshot_out"))),
        ]),
        "A failed or cancelled attempt has no output snapshot.",
    );
    check(
        builder,
        "check:terminal_failure_finding",
        E::And(vec![
            failed(),
            E::cmp(CmpOp::Eq, E::Col("finding_count"), E::Lit(Cell::U64(0))),
        ]),
        "An unsuccessful attempt retains at least one actual execution or rule finding.",
    );
    check(
        builder,
        "check:terminal_duration",
        E::cmp(CmpOp::Lt, E::Col("duration_ms"), E::Lit(Cell::F64(0.0))),
        "A terminal attempt duration is finite and nonnegative.",
    );
    let element = |name| E::Field {
        expr: Box::new(E::Col("finding")),
        name,
    };
    let invalid_origin = E::And(vec![
        E::IsNull(Box::new(element("check_id"))),
        E::Or(vec![
            E::Not(Box::new(failed())),
            E::cmp(
                CmpOp::NotEq,
                element("severity"),
                E::Lit(Cell::Enum("error")),
            ),
        ]),
    ]);
    let unnest = RulePlan::Unnest {
        input: Box::new(scan(RELATION, "subject")),
        column: "findings",
        value_name: "finding",
        null_list: NullListPolicy::Reject,
        empty_list: EmptyListPolicy::NoMembers,
    };
    invariant(
        builder,
        RELATION,
        "check:terminal_finding_origin",
        InvariantKind::Check,
        &["pass_run_id"],
        RulePlan::Distinct(Box::new(project(
            filter(unnest, invalid_origin),
            &["pass_run_id"],
        ))),
        "Only an execution error in a failed or cancelled attempt may omit a diagnostic check identity.",
    );
}
