// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declarative invariant constructors; no second row-wise implementation.
use crate::RegistryBuilder;
use crate::model::{
    AggregateEmptyPolicy, AggregateNullPolicy, InvariantDecl, InvariantKind, RuleAggregate,
    RuleAggregateFn, RuleDecl, RuleExpr, RuleHead, RulePlan,
};

pub(super) fn scan(relation: impl Into<String>, port: &'static str) -> RulePlan {
    RulePlan::Scan {
        relation: relation.into(),
        port,
    }
}
pub(super) fn project(input: RulePlan, keys: &[&'static str]) -> RulePlan {
    RulePlan::Project {
        input: Box::new(input),
        columns: (keys
            .iter()
            .map(|key| (*key, RuleExpr::col(*key)))
            .collect::<Vec<_>>())
        .into_iter()
        .map(|(name, expression)| (name.to_owned().into(), expression))
        .collect(),
    }
}
pub(super) fn filter(input: RulePlan, predicate: RuleExpr) -> RulePlan {
    RulePlan::Filter {
        input: Box::new(input),
        predicate,
    }
}
pub(super) fn count(output_name: &'static str) -> RuleAggregate {
    RuleAggregate {
        function: RuleAggregateFn::Count,
        input: None,
        output_name: (output_name).into(),
        order_by: vec![],
        null_policy: AggregateNullPolicy::Reject,
        empty_policy: AggregateEmptyPolicy::Zero,
    }
}
pub(super) fn declare(
    builder: &mut RegistryBuilder,
    relation: &str,
    name: &str,
    kind: InvariantKind,
    keys: &[&'static str],
    plan: RulePlan,
    doc: &'static str,
) {
    let rule_name = format!("{name}:{relation}");
    let rule = RuleDecl::new(
        rule_name.clone(),
        "1",
        1,
        RuleHead::Violations {
            of: relation.to_owned(),
            key_columns: keys.to_vec(),
        },
        plan,
    )
    .stratified_negation();
    builder.declare_rule(rule);
    builder.declare_invariant(InvariantDecl::error(
        relation,
        name,
        kind,
        format!("{rule_name}@1"),
        doc,
    ));
}
