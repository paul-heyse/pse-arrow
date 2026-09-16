// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact continuous-domain applicability and ordinal cardinality (ADR-0053 revision27).
use super::inv::{count, declare as invariant, filter, project, scan};
use crate::{
    RegistryBuilder,
    model::{Cell, CmpOp, InvariantKind, NullEquality, RuleExpr, RulePlan},
};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    let missing = RulePlan::AntiJoin {
        left: Box::new(filter(
            scan("authored.domains", "subject"),
            RuleExpr::col("continuous"),
        )),
        right: Box::new(scan("authored.continuous_domains", "details")),
        keys: (vec![("domain_id", "domain_id")])
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    };
    invariant(
        builder,
        "authored.domains",
        "cardinality:continuous_detail",
        InvariantKind::Cardinality,
        &["domain_id"],
        project(missing, &["domain_id"]),
        "Every continuous domain has one detail; its declared primary key enforces at most one.",
    );
    let domains = RulePlan::Project {
        input: Box::new(scan("authored.domains", "domains")),
        columns: (vec![
            ("__domain", RuleExpr::col("domain_id")),
            ("__continuous", RuleExpr::col("continuous")),
            ("__unit", RuleExpr::col("unit_id")),
        ])
        .into_iter()
        .map(|(name, expression)| (name.to_owned().into(), expression))
        .collect(),
    };
    let joined = RulePlan::EquiJoin {
        left: Box::new(scan("authored.continuous_domains", "subject")),
        right: Box::new(domains),
        keys: (vec![("domain_id", "__domain")])
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
        null_equality: NullEquality::NullEqualsNothing,
    };
    let invalid = filter(
        joined,
        RuleExpr::Or(vec![
            RuleExpr::Not(Box::new(RuleExpr::col("__continuous"))),
            RuleExpr::IsNull(Box::new(RuleExpr::col("__unit"))),
            RuleExpr::IsDistinctFrom(
                Box::new(RuleExpr::col("unit_id")),
                Box::new(RuleExpr::col("__unit")),
            ),
        ]),
    );
    invariant(
        builder,
        "authored.continuous_domains",
        "domain:continuous_context",
        InvariantKind::Domain,
        &["domain_id"],
        project(invalid, &["domain_id"]),
        "Only continuous domains carry a detail, with an explicit parent unit equal to the detail unit.",
    );
    ordinal_cardinality(builder);
}
fn ordinal_cardinality(builder: &mut RegistryBuilder) {
    let grouped = RulePlan::Aggregate {
        input: Box::new(scan("authored.domain_members", "members")),
        group: (vec!["domain_id", "ordinal"])
            .into_iter()
            .map(Into::into)
            .collect(),
        aggregates: vec![count("__count")],
    };
    let repeated = filter(
        grouped,
        RuleExpr::cmp(
            CmpOp::Gt,
            RuleExpr::col("__count"),
            RuleExpr::Lit(Cell::U64(1)),
        ),
    );
    let repeated = RulePlan::Project {
        input: Box::new(repeated),
        columns: (vec![
            ("__domain", RuleExpr::col("domain_id")),
            ("__ordinal", RuleExpr::col("ordinal")),
        ])
        .into_iter()
        .map(|(name, expression)| (name.to_owned().into(), expression))
        .collect(),
    };
    let offending = RulePlan::EquiJoin {
        left: Box::new(scan("authored.domain_members", "subject")),
        right: Box::new(repeated),
        keys: (vec![("domain_id", "__domain"), ("ordinal", "__ordinal")])
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
        null_equality: NullEquality::NullEqualsNothing,
    };
    invariant(
        builder,
        "authored.domain_members",
        "cardinality:member_ordinal",
        InvariantKind::Cardinality,
        &["member_id"],
        RulePlan::Distinct(Box::new(project(offending, &["member_id"]))),
        "Member ordinals identify exactly one member within each actual domain; every offending member is reported.",
    );
}
