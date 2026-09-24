// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual immutable function implementations, independent of names or hash claims.

use datafusion::common::{DataFusionError, Result, tree_node::TreeNodeRecursion};
use datafusion::execution::session_state::SessionState;
use datafusion::logical_expr::{
    AggregateUDF, Expr, HigherOrderUDF, LogicalPlan, ScalarUDF, WindowFunctionDefinition, WindowUDF,
};
use std::sync::Arc;

/// Apply command-time registration changes without rebuilding the state that
/// owns native SQL prepared definitions. Ordinary namespace/SET commands take
/// the unchanged-map path. Native registration APIs retain aliases and owners.
pub(super) fn apply_registry_changes(
    retained: &mut SessionState,
    completed: &SessionState,
) -> Result<bool> {
    use datafusion::logical_expr::registry::FunctionRegistry;
    let mut changed = false;
    macro_rules! sync_functions {
        ($get:ident, $register:ident, $remove:ident) => {{
            let removed: Vec<_> = retained
                .$get()
                .keys()
                .filter(|name| !completed.$get().contains_key(*name))
                .cloned()
                .collect();
            for name in removed {
                retained.$remove(&name)?;
                changed = true;
            }
            for (name, function) in completed.$get() {
                if !retained
                    .$get()
                    .get(name)
                    .is_some_and(|old| Arc::ptr_eq(old, function))
                {
                    retained.$register(function.clone())?;
                    changed = true;
                }
            }
            if retained.$get().len() != completed.$get().len()
                || !completed.$get().iter().all(|(name, function)| {
                    retained
                        .$get()
                        .get(name)
                        .is_some_and(|old| Arc::ptr_eq(old, function))
                })
            {
                return Err(DataFusionError::Plan(
                    "command function aliases cannot preserve the exact native registry".into(),
                ));
            }
        }};
    }
    sync_functions!(scalar_functions, register_udf, deregister_udf);
    sync_functions!(aggregate_functions, register_udaf, deregister_udaf);
    sync_functions!(window_functions, register_udwf, deregister_udwf);
    sync_functions!(
        higher_order_functions,
        register_higher_order_function,
        deregister_higher_order_function
    );
    let removed: Vec<_> = retained
        .table_functions()
        .keys()
        .filter(|name| !completed.table_functions().contains_key(*name))
        .cloned()
        .collect();
    for name in removed {
        retained.deregister_udtf(&name)?;
        changed = true;
    }
    for (name, function) in completed.table_functions() {
        if !retained
            .table_functions()
            .get(name)
            .is_some_and(|old| Arc::ptr_eq(old.function(), function.function()))
        {
            retained.register_udtf(name, function.function().clone());
            changed = true;
        }
    }
    Ok(changed)
}

#[derive(Debug)]
pub(super) struct Functions {
    scalar: Vec<Arc<ScalarUDF>>,
    aggregate: Vec<Arc<AggregateUDF>>,
    window: Vec<Arc<WindowUDF>>,
    higher_order: Vec<Arc<HigherOrderUDF>>,
}
impl Functions {
    pub(super) fn selected(
        &self,
        plan: &LogicalPlan,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self> {
        let mut selected = Self {
            scalar: vec![],
            aggregate: vec![],
            window: vec![],
            higher_order: vec![],
        };
        super::traversal::visit(
            plan,
            super::traversal::Purpose::Evidence,
            pool,
            cancel,
            |node, _| {
                node.apply_expressions(|root| {
                    super::traversal::expression(root, |expr| {
                        match expr {
                            Expr::ScalarFunction(call) if self.scalar(&call.func).is_ok() => {
                                let function = super::field_transfer::native_scalar(&call.func)
                                    .unwrap_or(&call.func);
                                if !selected
                                    .scalar
                                    .iter()
                                    .any(|known| Arc::ptr_eq(known.inner(), function.inner()))
                                {
                                    selected.scalar.push(function.clone());
                                }
                            }
                            Expr::AggregateFunction(call) if self.aggregate(&call.func).is_ok() => {
                                let function =
                                    super::aggregate::native(&call.func).unwrap_or(&call.func);
                                if !selected
                                    .aggregate
                                    .iter()
                                    .any(|known| Arc::ptr_eq(known.inner(), function.inner()))
                                {
                                    selected.aggregate.push(function.clone());
                                }
                            }
                            Expr::WindowFunction(call) => match &call.fun {
                                WindowFunctionDefinition::AggregateUDF(function)
                                    if self.aggregate(function).is_ok() =>
                                {
                                    selected.aggregate.push(function.clone());
                                }
                                WindowFunctionDefinition::WindowUDF(function)
                                    if self.window(function).is_ok() =>
                                {
                                    selected.window.push(function.clone());
                                }
                                _ => {}
                            },
                            Expr::HigherOrderFunction(call)
                                if self.higher_order(&call.func).is_ok() =>
                            {
                                selected.higher_order.push(call.func.clone());
                            }
                            _ => {}
                        }
                        Ok(TreeNodeRecursion::Continue)
                    })
                })?;
                Ok(TreeNodeRecursion::Continue)
            },
        )?;
        Ok(selected)
    }
    pub(super) fn remains_in(&self, functions: &Self) -> bool {
        self.scalar
            .iter()
            .all(|function| functions.scalar(function).is_ok())
            && self
                .aggregate
                .iter()
                .all(|function| functions.aggregate(function).is_ok())
            && self
                .window
                .iter()
                .all(|function| functions.window(function).is_ok())
            && self
                .higher_order
                .iter()
                .all(|function| functions.higher_order(function).is_ok())
    }
    pub(super) fn from_state(state: &SessionState) -> Arc<Self> {
        Arc::new(Self {
            scalar: state.scalar_functions().values().cloned().collect(),
            aggregate: state.aggregate_functions().values().cloned().collect(),
            window: state.window_functions().values().cloned().collect(),
            higher_order: state.higher_order_functions().values().cloned().collect(),
        })
    }
    pub(super) fn scalar(&self, value: &ScalarUDF) -> Result<()> {
        let value = super::field_transfer::native_scalar(value).map_or(value, AsRef::as_ref);
        check(
            self.scalar
                .iter()
                .any(|actual| Arc::ptr_eq(actual.inner(), value.inner())),
            "scalar",
            value.name(),
        )
    }
    pub(super) fn aggregate(&self, value: &AggregateUDF) -> Result<()> {
        let value = super::aggregate::native(value).map_or(value, AsRef::as_ref);
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
    pub(super) fn admit_plan(
        &self,
        plan: &LogicalPlan,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<()> {
        super::traversal::visit(
            plan,
            super::traversal::Purpose::Evidence,
            pool,
            cancel,
            |node, _| {
                node.apply_expressions(|expression| {
                    super::traversal::expression(expression, |expression| {
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
                    Ok(TreeNodeRecursion::Continue)
                })?;
                // Selected provider descriptors retain their actual private view
                // implementations. Scan filters are serialized above and checked;
                // the provider's implementation is admitted by source ownership.
                Ok(if matches!(node, LogicalPlan::TableScan(_)) {
                    TreeNodeRecursion::Jump
                } else {
                    TreeNodeRecursion::Continue
                })
            },
        )?;
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
