// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Iterative requirement placement over native owners and lexical regions.
use super::{ExecutionContract, Requirement, invalid};
use crate::session::{
    admission::{NodeIdentity, identity},
    cache,
};
use datafusion::{
    common::{
        Result,
        tree_node::{Transformed, TreeNode, TreeNodeRecursion},
    },
    logical_expr::LogicalPlan,
};

use pse_columnar::{CancellationToken, MemoryConsumer, MemoryPool};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key(NodeIdentity, usize, bool);
#[derive(Default)]
struct Region {
    requirements: Vec<Requirement>,
    shared: HashMap<NodeIdentity, LogicalPlan>,
}
enum Work {
    Enter(LogicalPlan, Key),
    Exit(LogicalPlan, Key, Vec<Option<Key>>),
}

/// Preserve required work separately in each subquery/cache region, without
/// recursively walking or hashing complete native plan trees.
#[expect(
    clippy::too_many_lines,
    reason = "one iterative state machine preserves native lexical and effect regions"
)]
pub(in crate::session) fn protect(
    plan: LogicalPlan,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<LogicalPlan> {
    // Retain the original graph so addresses used by the temporary index cannot
    // be recycled. These are actual ownership keys, never semantic identities.
    let original = plan;
    let root = Key(identity(&original), 0, true);
    let reservation = MemoryConsumer::new("session:contract-protection").register(pool);
    let mut regions = vec![Region::default()];
    let mut work = vec![Work::Enter(original.clone(), root)];
    let mut active = HashSet::new();
    let mut done = HashMap::<Key, Transformed<LogicalPlan>>::new();
    while let Some(task) = work.pop() {
        cancel.checkpoint().map_err(pse_columnar::external)?;
        match task {
            Work::Enter(plan, key) => {
                if done.contains_key(&key) {
                    continue;
                }
                if !active.insert(key) {
                    return Err(invalid("cyclic contract plan"));
                }
                reservation.try_grow(size_of::<Work>() + 512)?;
                if boundary(&plan) {
                    active.remove(&key);
                    done.insert(key, Transformed::no(plan));
                    continue;
                }
                let mut edges = Vec::new();
                let mut children = Vec::new();
                plan.apply_subqueries(|wrapper| {
                    let LogicalPlan::Subquery(query) = wrapper else {
                        return Err(invalid("expected native subquery"));
                    };
                    let region = regions.len();
                    regions.push(Region::default());
                    let child = Key(identity(&query.subquery), region, true);
                    edges.push(Some(child));
                    children.push(Work::Enter(query.subquery.as_ref().clone(), child));
                    Ok(TreeNodeRecursion::Continue)
                })?;
                let operation = match &plan {
                    LogicalPlan::Extension(extension) => extension
                        .node
                        .as_any()
                        .downcast_ref::<crate::operation::Operation>(
                    ),
                    _ => None,
                };
                for (port, input) in plan.inputs().into_iter().enumerate() {
                    if operation.is_some_and(|operation| operation.definition_only(port)) {
                        edges.push(None);
                    } else {
                        let child = Key(
                            identity(input),
                            key.1,
                            matches!(plan, LogicalPlan::Explain(_)),
                        );
                        edges.push(Some(child));
                        children.push(Work::Enter(input.clone(), child));
                    }
                }
                reservation.try_grow(edges.len() * (size_of::<Work>() + size_of::<Key>()))?;
                work.push(Work::Exit(plan, key, edges));
                work.extend(children.into_iter().rev());
            }
            Work::Exit(original, key, edges) => {
                let mut edges = edges.into_iter();
                let mut result = original
                    .clone()
                    .map_subqueries(|wrapper| {
                        let LogicalPlan::Subquery(mut query) = wrapper else {
                            return Err(invalid("expected native subquery"));
                        };
                        let child = result(&done, edges.next().flatten())?;
                        if child.transformed {
                            query.subquery = Arc::new(child.data.clone());
                        }
                        Ok(Transformed::new_transformed(
                            LogicalPlan::Subquery(query),
                            child.transformed,
                        ))
                    })?
                    .transform_data(|plan| {
                        plan.map_children(|input| {
                            let Some(key) = edges.next().flatten() else {
                                return Ok(Transformed::no(input));
                            };
                            let child = result(&done, Some(key))?;
                            Ok(Transformed::new_transformed(
                                child.data.clone(),
                                child.transformed,
                            ))
                        })
                    })?;
                let region = &mut regions[key.1];
                if let LogicalPlan::Extension(extension) = &result.data
                    && let Some(contract) =
                        extension.node.as_any().downcast_ref::<ExecutionContract>()
                {
                    let LogicalPlan::Extension(original_extension) = &original else {
                        return Err(invalid("contract owner changed"));
                    };
                    let original_contract = original_extension
                        .node
                        .as_any()
                        .downcast_ref::<ExecutionContract>()
                        .ok_or_else(|| invalid("contract owner changed"))?;
                    let mut contract = contract.clone();
                    for (requirement, original) in contract
                        .requirements
                        .iter_mut()
                        .zip(&original_contract.requirements)
                    {
                        requirement.plan = region
                            .shared
                            .entry(identity(&original.plan))
                            .or_insert_with(|| cache::share_required(requirement.plan.clone()))
                            .clone();
                        region.requirements.push(requirement.clone());
                    }
                    contract.protected = true;
                    result = Transformed::yes(contract.into_plan());
                } else if !key.2 && command(&result.data) {
                    let shared = cache::share_required(result.data);
                    region.requirements.push(Requirement {
                        plan: shared.clone(),
                        check: false,
                    });
                    result = Transformed::yes(shared);
                }
                if key.2 && !region.requirements.is_empty() {
                    result = Transformed::yes(
                        ExecutionContract {
                            input: result.data,
                            requirements: std::mem::take(&mut region.requirements),
                            effects: BTreeSet::new(),
                            protected: true,
                        }
                        .into_plan(),
                    );
                }
                active.remove(&key);
                done.insert(key, result);
            }
        }
    }
    done.remove(&root)
        .map(|value| value.data)
        .ok_or_else(|| invalid("contract root absent"))
}
fn result(
    done: &HashMap<Key, Transformed<LogicalPlan>>,
    key: Option<Key>,
) -> Result<&Transformed<LogicalPlan>> {
    key.and_then(|key| done.get(&key))
        .ok_or_else(|| invalid("contract child absent"))
}
fn boundary(plan: &LogicalPlan) -> bool {
    cache::is_boundary(plan)
        || matches!(plan, LogicalPlan::Extension(extension)
        if extension.node.as_any().downcast_ref::<ExecutionContract>().is_some_and(|contract| contract.protected))
}
fn command(plan: &LogicalPlan) -> bool {
    matches!(plan, LogicalPlan::Dml(_) | LogicalPlan::Copy(_))
        || matches!(plan, LogicalPlan::Extension(extension)
            if extension.node.as_any().downcast_ref::<crate::operation::Operation>()
                .is_some_and(|operation| operation.family() == crate::operation::Family::Command))
}
