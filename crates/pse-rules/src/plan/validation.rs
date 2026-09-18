// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fixed-point policy is checked against native query operators and selected inputs.
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion_common::{
    JoinType,
    tree_node::{TreeNode, TreeNodeRecursion},
};
use datafusion_expr::LogicalPlan;
use pse_schema::{
    Registry,
    model::{DependencyMode, NegationPolicy, RuleSpec},
};

pub(super) fn validate(
    plan: &LogicalPlan,
    rule: &RuleSpec,
    registry: &Registry,
) -> Result<(), RuleError> {
    let evolving = registry
        .rules()
        .iter()
        .filter(|other| other.stratum == rule.stratum)
        .map(|other| other.head.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    visit(plan, rule, &evolving, false, false)
}
fn visit(
    plan: &LogicalPlan,
    rule: &RuleSpec,
    evolving: &std::collections::BTreeSet<&str>,
    negative: bool,
    nonmonotone: bool,
) -> Result<(), RuleError> {
    if let LogicalPlan::TableScan(scan) = plan {
        let name = format!(
            "{}.{}",
            scan.table_name.schema().unwrap_or_default(),
            scan.table_name.table()
        );
        let mode = if negative {
            DependencyMode::Negate
        } else {
            DependencyMode::Read
        };
        if !rule
            .inputs
            .iter()
            .any(|input| input.relation == name && input.mode == mode)
        {
            return Err(internal(format!(
                "native dependency {name} ({mode}) is absent from {}",
                rule.qualified_name()
            )));
        }
        if negative && rule.negation != NegationPolicy::Stratified {
            return Err(internal("native negation requires a settled stratum"));
        }
        if evolving.contains(name.as_str()) && (negative || nonmonotone || !rule.monotonic) {
            return Err(internal(format!(
                "native operator is not monotone over evolving relation {name}"
            )));
        }
    }
    let nonmonotone = nonmonotone
        || matches!(
            plan,
            LogicalPlan::Aggregate(_) | LogicalPlan::Window(_) | LogicalPlan::Limit(_)
        )
        || matches!(plan, LogicalPlan::Join(join) if matches!(join.join_type, JoinType::Left | JoinType::Right | JoinType::Full));
    for (index, input) in plan.inputs().into_iter().enumerate() {
        let negative = negative
            || matches!(plan,LogicalPlan::Join(join) if (join.join_type==JoinType::LeftAnti && index==1)||(join.join_type==JoinType::RightAnti && index==0));
        visit(input, rule, evolving, negative, nonmonotone)?;
    }
    // Scalar and predicate subqueries are native plans too. Traverse them rather
    // than letting a hidden dependency bypass stratum checks.
    for expression in plan.expressions() {
        expression
            .apply(|expr| {
                let subquery = match expr {
                    datafusion_expr::Expr::Exists(value) => Some((&value.subquery, value.negated)),
                    datafusion_expr::Expr::InSubquery(value) => {
                        Some((&value.subquery, value.negated))
                    }
                    datafusion_expr::Expr::ScalarSubquery(value) => Some((value, false)),
                    _ => None,
                };
                if let Some((query, negated)) = subquery {
                    visit(&query.subquery, rule, evolving, negative || negated, true).map_err(
                        |error| datafusion_common::DataFusionError::External(Box::new(error)),
                    )?;
                }
                Ok(TreeNodeRecursion::Continue)
            })
            .map_err(engine)?;
    }
    Ok(())
}
