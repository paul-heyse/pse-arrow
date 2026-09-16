// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual immutable function implementations, independent of names or hash claims.

use datafusion::common::{
    DataFusionError, Result,
    tree_node::{TreeNode, TreeNodeRecursion},
};
use datafusion::execution::session_state::SessionState;
use datafusion::logical_expr::{
    AggregateUDF, Expr, HigherOrderUDF, LogicalPlan, ScalarUDF, WindowFunctionDefinition, WindowUDF,
};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct Functions {
    scalar: Vec<Arc<ScalarUDF>>,
    aggregate: Vec<Arc<AggregateUDF>>,
    window: Vec<Arc<WindowUDF>>,
    higher_order: Vec<Arc<HigherOrderUDF>>,
}
impl Functions {
    pub(super) fn from_state(state: &SessionState) -> Arc<Self> {
        Arc::new(Self {
            scalar: state.scalar_functions().values().cloned().collect(),
            aggregate: state.aggregate_functions().values().cloned().collect(),
            window: state.window_functions().values().cloned().collect(),
            higher_order: state.higher_order_functions().values().cloned().collect(),
        })
    }
    pub(super) fn scalar(&self, value: &ScalarUDF) -> Result<()> {
        check(
            self.scalar
                .iter()
                .any(|actual| Arc::ptr_eq(actual.inner(), value.inner())),
            "scalar",
            value.name(),
        )
    }
    pub(super) fn aggregate(&self, value: &AggregateUDF) -> Result<()> {
        check(
            self.aggregate
                .iter()
                .any(|actual| Arc::ptr_eq(actual.inner(), value.inner())),
            "aggregate",
            value.name(),
        )
    }
    pub(super) fn window(&self, value: &WindowUDF) -> Result<()> {
        check(
            self.window
                .iter()
                .any(|actual| Arc::ptr_eq(actual.inner(), value.inner())),
            "window",
            value.name(),
        )
    }
    pub(super) fn higher_order(&self, value: &HigherOrderUDF) -> Result<()> {
        check(
            self.higher_order
                .iter()
                .any(|actual| Arc::ptr_eq(actual.inner(), value.inner())),
            "higher-order",
            value.name(),
        )
    }
    pub(super) fn admit_plan(&self, plan: &LogicalPlan) -> Result<()> {
        plan.apply_with_subqueries(|node| {
            for expression in node.expressions() {
                expression.apply(|expression| {
                    match expression {
                        Expr::ScalarFunction(call) => self.scalar(&call.func)?,
                        Expr::AggregateFunction(call) => self.aggregate(&call.func)?,
                        Expr::HigherOrderFunction(call) => self.higher_order(&call.func)?,
                        Expr::WindowFunction(call) => match &call.fun {
                            WindowFunctionDefinition::AggregateUDF(function) => {
                                self.aggregate(function)?;
                            }
                            WindowFunctionDefinition::WindowUDF(function) => {
                                self.window(function)?;
                            }
                        },
                        _ => {}
                    }
                    Ok(TreeNodeRecursion::Continue)
                })?;
            }
            Ok(TreeNodeRecursion::Continue)
        })?;
        Ok(())
    }
}
fn check(valid: bool, kind: &str, name: &str) -> Result<()> {
    if valid {
        Ok(())
    } else {
        Err(DataFusionError::Plan(format!(
            "{kind} function {name} is outside the retained implementation inventory"
        )))
    }
}
