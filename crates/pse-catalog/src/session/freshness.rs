// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Classify actual native owners without cloning expressions or expanding shared
//! extensions repeatedly. Contracts remain scoped to each root; a contract in a
//! sibling output cannot qualify an otherwise uncontracted operation.
use super::{
    SnapshotSession,
    admission::{NodeIdentity, identity},
    contract::ExecutionContract,
};
use datafusion::{
    common::{
        DataFusionError, Result,
        tree_node::{TreeNode, TreeNodeRecursion},
    },
    logical_expr::{Expr, LogicalPlan, Volatility},
};
use pse_ids::CancellationToken;
use pse_schema::model::provider::OperationEffect;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

impl SnapshotSession {
    pub(crate) fn plans_require_fresh<'a>(
        &self,
        plans: impl IntoIterator<Item = &'a LogicalPlan>,
        cancel: &CancellationToken,
    ) -> Result<bool> {
        let mut reservation = self.reserver.open("session:freshness-discovery");
        check(
            plans,
            |bytes| {
                reservation.try_grow(bytes).map_err(|error| {
                    DataFusionError::External(Box::new(crate::CatalogError::from(error)))
                })
            },
            cancel,
        )
    }
}

#[derive(Clone, Copy)]
struct Facts {
    varying: bool,
    extension: Option<usize>,
    contracted: Option<usize>,
}

pub(super) fn check<'a>(
    plans: impl IntoIterator<Item = &'a LogicalPlan>,
    mut charge: impl FnMut(usize) -> Result<()>,
    cancel: &CancellationToken,
) -> Result<bool> {
    let mut facts = HashMap::<NodeIdentity, Facts>::new();
    let mut visited = HashSet::new();
    let mut extensions = HashSet::new();
    let mut contracted = HashSet::new();
    let mut root_capacity = 0;
    for plan in plans {
        visited.clear();
        extensions.clear();
        contracted.clear();
        let mut varying = false;
        plan.apply_with_subqueries(|node| {
            cancel.checkpoint().map_err(|error| {
                DataFusionError::External(Box::new(crate::CatalogError::from(error)))
            })?;
            // Subquery wrappers are temporary. Their actual inputs and function
            // expressions are still traversed using the native contract.
            if matches!(node, LogicalPlan::Subquery(_)) {
                return Ok(TreeNodeRecursion::Continue);
            }
            let key = identity(node);
            if visited.contains(&key) {
                return Ok(TreeNodeRecursion::Jump);
            }
            if visited.len() == root_capacity {
                charge(256)?;
                root_capacity += 1;
            }
            visited.insert(key);
            let value = if let Some(value) = facts.get(&key) {
                *value
            } else {
                charge(128)?;
                let value = inspect(node)?;
                facts.insert(key, value);
                value
            };
            varying |= value.varying;
            extensions.extend(value.extension);
            contracted.extend(value.contracted);
            Ok(if varying {
                TreeNodeRecursion::Stop
            } else {
                TreeNodeRecursion::Continue
            })
        })?;
        if varying || extensions.difference(&contracted).next().is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn inspect(node: &LogicalPlan) -> Result<Facts> {
    let mut result = Facts {
        varying: false,
        extension: None,
        contracted: None,
    };
    if let LogicalPlan::Extension(extension) = node {
        if let Some(contract) = extension.node.as_any().downcast_ref::<ExecutionContract>() {
            result.varying |= contract
                .effects()
                .iter()
                .any(|effect| *effect != OperationEffect::Read);
            if let LogicalPlan::Extension(operation) = contract.operation() {
                result.contracted = Some(Arc::as_ptr(&operation.node).cast::<()>() as usize);
            }
        } else {
            result.extension = Some(Arc::as_ptr(&extension.node).cast::<()>() as usize);
        }
    }
    node.apply_expressions(|expression| {
        expression.apply(|expression| {
            result.varying |= match expression {
                Expr::ScalarFunction(value) => {
                    value.func.signature().volatility != Volatility::Immutable
                }
                Expr::AggregateFunction(value) => {
                    value.func.signature().volatility != Volatility::Immutable
                }
                Expr::WindowFunction(value) => {
                    value.fun.signature().volatility != Volatility::Immutable
                }
                Expr::HigherOrderFunction(value) => {
                    value.func.signature().volatility != Volatility::Immutable
                }
                Expr::ScalarVariable(..) => true,
                _ => false,
            };
            Ok(if result.varying {
                TreeNodeRecursion::Stop
            } else {
                TreeNodeRecursion::Continue
            })
        })
    })?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::logical_expr::{LogicalPlanBuilder, Union};

    #[test]
    fn sibling_outputs_share_only_native_facts_and_keep_bounded_scratch() {
        let mut plan = LogicalPlanBuilder::empty(true).build().unwrap();
        for _ in 0..18 {
            let input = Arc::new(ExecutionContract::plan(
                plan,
                None,
                [OperationEffect::Read].into_iter().collect(),
            ));
            plan = LogicalPlan::Union(Union {
                inputs: vec![Arc::clone(&input), Arc::clone(&input)],
                schema: Arc::clone(input.schema()),
            });
        }
        let roots = vec![plan; 128];
        let cancel = CancellationToken::new();
        let mut charged = 0;
        assert!(
            !check(
                roots.iter(),
                |bytes| {
                    charged += bytes;
                    Ok(())
                },
                &cancel
            )
            .unwrap()
        );
        // 128 distinct Union root values, with all their actual shared children.
        // Growth follows unique native owners, not the expanded binary tree or
        // a fresh scratch allocation for each of the 128 output roots.
        assert!(charged < 32 << 10, "actual charged scratch: {charged}");
        cancel.cancel();
        assert!(check(roots.iter(), |_| Ok(()), &cancel).is_err());
    }
}
