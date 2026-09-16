// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual declaration identity for private intermediate reuse. Numeric equality
//! alone cannot equate different floating literal representations such as signed zero.

use pse_schema::model::{Cell, RuleExpr, RulePlan};

pub(super) fn same_plan(left: &RulePlan, right: &RulePlan) -> bool {
    if left != right {
        return false;
    }
    let mut left_bits = vec![];
    let mut right_bits = vec![];
    floating_literals(left, &mut left_bits);
    floating_literals(right, &mut right_bits);
    left_bits == right_bits
}

pub(super) fn same_literal(left: Option<&Cell>, right: Option<&Cell>) -> bool {
    if left != right {
        return false;
    }
    let mut left_bits = vec![];
    let mut right_bits = vec![];
    if let Some(value) = left {
        cell(value, &mut left_bits);
    }
    if let Some(value) = right {
        cell(value, &mut right_bits);
    }
    left_bits == right_bits
}

fn floating_literals(plan: &RulePlan, values: &mut Vec<u64>) {
    match plan {
        RulePlan::Filter { predicate, .. } | RulePlan::Assert { predicate, .. } => {
            expression(predicate, values);
        }
        RulePlan::Project { columns, .. } => {
            for (_, value) in columns {
                expression(value, values);
            }
        }
        RulePlan::Aggregate { aggregates, .. } => {
            for value in aggregates.iter().filter_map(|value| value.input.as_ref()) {
                expression(value, values);
            }
        }
        RulePlan::Scan { .. }
        | RulePlan::RecursiveRef { .. }
        | RulePlan::EquiJoin { .. }
        | RulePlan::AntiJoin { .. }
        | RulePlan::Union(_)
        | RulePlan::Distinct(_)
        | RulePlan::Unnest { .. }
        | RulePlan::Recursive { .. } => {}
    }
    for child in plan.children() {
        floating_literals(child, values);
    }
}

fn expression(expr: &RuleExpr, values: &mut Vec<u64>) {
    match expr {
        RuleExpr::Col(_) => {}
        RuleExpr::Lit(value) => cell(value, values),
        RuleExpr::Call { args, .. } | RuleExpr::And(args) | RuleExpr::Or(args) => {
            for argument in args {
                expression(argument, values);
            }
        }
        RuleExpr::InList { expr, list } => {
            expression(expr, values);
            for value in list {
                cell(value, values);
            }
        }
        RuleExpr::Not(expr)
        | RuleExpr::IsNull(expr)
        | RuleExpr::IsNotNull(expr)
        | RuleExpr::ListLen(expr)
        | RuleExpr::IsTrue(expr)
        | RuleExpr::IsFalse(expr)
        | RuleExpr::IsUnknown(expr)
        | RuleExpr::Field { expr, .. } => expression(expr, values),
        RuleExpr::Cmp { l, r, .. }
        | RuleExpr::IsDistinctFrom(l, r)
        | RuleExpr::IsNotDistinctFrom(l, r) => {
            expression(l, values);
            expression(r, values);
        }
    }
}

fn cell(value: &Cell, values: &mut Vec<u64>) {
    match value {
        Cell::F64(value) => values.push(value.to_bits()),
        Cell::List(children) | Cell::Struct(children) => {
            for child in children {
                cell(child, values);
            }
        }
        _ => {}
    }
}
