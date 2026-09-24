// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Scoped control tables over retained native plans, not a second plan language.
use super::admission::{NodeIdentity, identity};
use datafusion::{
    arrow::datatypes::SchemaRef,
    common::{
        DataFusionError, Result,
        tree_node::{Transformed, TreeNode, TreeNodeRecursion},
    },
    logical_expr::{Expr, LogicalPlan},
};

use pse_columnar::{CancellationToken, MemoryPool};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Purpose {
    Rewrite,
    Evidence,
    Observation,
    Freshness,
}
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub(crate) struct Scope {
    pub worktables: Vec<(String, SchemaRef)>,
    pub outer: Vec<Expr>,
    lexical: Vec<(NodeIdentity, usize)>,
    pub qualified: Vec<usize>,
    pub unbound: bool,
}
impl Scope {
    fn extent(&self) -> usize {
        256 + self.lexical.len() * 32 + self.worktables.len() * 128 + self.outer.len() * 256
    }
}
pub(crate) struct Node {
    /// Identity of the retained original, independent of its lexical uses.
    pub owner: NodeIdentity,
    pub plan: LogicalPlan,
    pub scope: Scope,
    // Native subquery order followed by ordinary input order.
    pub children: Vec<usize>,
    pub subqueries: usize,
}
pub(crate) struct Graph {
    pub nodes: Vec<Node>,
    pub roots: Vec<usize>,
    pub postorder: Vec<usize>,
}
fn invalid(message: &str) -> DataFusionError {
    DataFusionError::Plan(message.into())
}
fn checkpoint(cancel: &CancellationToken) -> Result<()> {
    cancel
        .checkpoint()
        .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))
}
fn admit_expressions(
    plan: &LogicalPlan,
    cancel: &CancellationToken,
    charge: &mut impl FnMut(usize) -> Result<()>,
) -> Result<()> {
    plan.apply_expressions(|expr| {
        expression(expr, |_| {
            checkpoint(cancel)?;
            charge(size_of::<Expr>() + 64)?;
            Ok(TreeNodeRecursion::Continue)
        })
    })?;
    Ok(())
}
/// Iterative expression enumeration. A depth ceiling protects native reconstruction,
/// which recursively clones expressions; a rejected expression is never partly admitted.
pub(crate) fn expression<'a>(
    root: &'a Expr,
    mut f: impl FnMut(&'a Expr) -> Result<TreeNodeRecursion>,
) -> Result<TreeNodeRecursion> {
    let mut stack = vec![(root, 0)];
    while let Some((expr, depth)) = stack.pop() {
        if depth > 128 {
            return Err(invalid(
                "native expression nesting exceeds admission depth 128",
            ));
        }
        match f(expr)? {
            TreeNodeRecursion::Stop => return Ok(TreeNodeRecursion::Stop),
            TreeNodeRecursion::Jump => continue,
            TreeNodeRecursion::Continue => {}
        }
        let start = stack.len();
        expr.apply_children(|child| {
            stack.push((child, depth + 1));
            Ok(TreeNodeRecursion::Continue)
        })?;
        stack[start..].reverse();
    }
    Ok(TreeNodeRecursion::Continue)
}

#[expect(
    clippy::too_many_lines,
    reason = "one iterative state machine owns scoped keys, native edges and postorder for all traversal purposes"
)]
pub(crate) fn graph<'a>(
    plans: impl IntoIterator<Item = &'a LogicalPlan>,
    purpose: Purpose,
    cancel: &CancellationToken,
    mut charge: impl FnMut(usize) -> Result<()>,
    mut enter: impl FnMut(&LogicalPlan, &Scope) -> Result<TreeNodeRecursion>,
) -> Result<Graph> {
    let mut result = Graph {
        nodes: vec![],
        roots: vec![],
        postorder: vec![],
    };
    let mut seen = HashMap::<(NodeIdentity, Scope), usize>::new();
    let mut expressions = std::collections::HashSet::new();
    let mut finished = std::collections::HashSet::new();
    let mut depths = Vec::<usize>::new();
    for root in plans {
        let root_key = (identity(root), Scope::default());
        if let Some(id) = seen.get(&root_key) {
            result.roots.push(*id);
            continue;
        }
        charge(size_of::<LogicalPlan>() + 256)?;
        // Root and child owners remain retained until every rewrite is complete.
        let mut stack = vec![(root.clone(), root_key, None::<usize>, false)];
        while let Some((plan, key, parent, exit)) = stack.pop() {
            checkpoint(cancel)?;
            if exit {
                let id = seen[&key];
                let depth = result.nodes[id]
                    .children
                    .iter()
                    .map(|child| depths[*child] + 1)
                    .max()
                    .unwrap_or(0);
                if purpose == Purpose::Rewrite && depth > 512 {
                    return Err(DataFusionError::ResourcesExhausted(
                        "native recursive rewrite nesting exceeds 512 plan edges".into(),
                    ));
                }
                depths[id] = depth;
                finished.insert(id);
                result.postorder.push(id);
                continue;
            }
            if let Some(id) = seen.get(&key).copied() {
                if !finished.contains(&id) {
                    return Err(invalid("cyclic native plan ownership"));
                }
                if let Some(parent) = parent {
                    result.nodes[parent].children.push(id);
                }
                continue;
            }
            let id = result.nodes.len();
            if expressions.insert(key.0) {
                admit_expressions(&plan, cancel, &mut charge)?;
            }
            charge(size_of::<Node>() + key.1.extent())?;
            seen.insert(key.clone(), id);
            depths.push(0);
            if let Some(parent) = parent {
                result.nodes[parent].children.push(id);
            } else {
                result.roots.push(id);
            }
            let action = enter(&plan, &key.1)?;
            result.nodes.push(Node {
                owner: key.0,
                plan: plan.clone(),
                scope: key.1.clone(),
                children: vec![],
                subqueries: 0,
            });
            if action == TreeNodeRecursion::Stop {
                break;
            }
            stack.push((plan.clone(), key.clone(), None, true));
            if action == TreeNodeRecursion::Jump {
                continue;
            }
            // A proved closed producer has no dependencies on the enclosing
            // subquery/worktable bindings. Its descendants are intrinsic to it.
            let mut scope = if super::cache::closed_input(&plan) {
                Scope::default()
            } else {
                key.1.clone()
            };
            // A contract qualifies its actual operation edge, not arbitrary
            // descendants. Ancestor paths are not intrinsic producer inputs.
            scope.qualified.clear();
            if purpose == Purpose::Freshness
                && let LogicalPlan::Extension(extension) = &plan
                && let Some(contract) = extension
                    .node
                    .as_any()
                    .downcast_ref::<super::contract::ExecutionContract>()
                && let LogicalPlan::Extension(operation) = contract.operation()
            {
                scope
                    .qualified
                    .push(Arc::as_ptr(&operation.node).cast::<()>() as usize);
            }
            let mut children = Vec::new();
            plan.apply_subqueries(|wrapper| {
                let LogicalPlan::Subquery(query) = wrapper else {
                    return Err(invalid("native subquery wrapper absent"));
                };
                // Base key/stack slots are included in the node allowance.
                // Charge the pending native enum and cloned scope vectors here.
                charge(size_of::<LogicalPlan>() + scope.extent() - 256)?;
                let mut child_scope = scope.clone();
                child_scope.unbound |= query.outer_ref_columns.iter().any(|reference| {
                    let Expr::OuterReferenceColumn(_, column) = reference else {
                        return true;
                    };
                    !scope.outer.contains(reference)
                        && plan.schema().index_of_column(column).is_err()
                        && plan
                            .inputs()
                            .iter()
                            .all(|input| input.schema().index_of_column(column).is_err())
                });
                child_scope
                    .outer
                    .extend(query.outer_ref_columns.iter().cloned());
                child_scope.lexical.push((key.0, children.len()));
                children.push((
                    query.subquery.as_ref().clone(),
                    (
                        identity(&query.subquery),
                        boundary_scope(&query.subquery, purpose, child_scope),
                    ),
                    Some(id),
                    false,
                ));
                Ok(TreeNodeRecursion::Continue)
            })?;
            result.nodes[id].subqueries = children.len();
            let inputs = if purpose == Purpose::Rewrite {
                plan.inputs()
            } else {
                super::cache::evidence_input(&plan)
                    .map_or_else(|| plan.inputs(), |input| vec![input])
            };
            for (ordinal, input) in inputs.into_iter().enumerate() {
                // Base key/stack slots are included in the node allowance.
                // Charge the pending native enum and cloned scope vectors here.
                charge(size_of::<LogicalPlan>() + scope.extent() - 256)?;
                let mut child_scope = scope.clone();
                if let LogicalPlan::RecursiveQuery(query) = &plan
                    && ordinal == 1
                {
                    child_scope.worktables.push((
                        query.name.clone(),
                        query.static_term.schema().inner().clone(),
                    ));
                    child_scope.lexical.push((key.0, ordinal));
                }
                children.push((
                    input.clone(),
                    (identity(input), boundary_scope(input, purpose, child_scope)),
                    Some(id),
                    false,
                ));
            }
            stack.extend(children.into_iter().rev());
        }
    }
    tracing::debug!(target: "pse::admission", scoped_nodes = result.nodes.len(), intrinsic_nodes = expressions.len(), roots = result.roots.len(), "native traversal");
    Ok(result)
}
fn boundary_scope(plan: &LogicalPlan, purpose: Purpose, scope: Scope) -> Scope {
    if purpose != Purpose::Freshness && super::cache::closed_input(plan) {
        Scope::default()
    } else {
        scope
    }
}

/// Native reconstruction with scope-qualified memoization; callbacks never see a
/// synthetic subquery wrapper as a reusable owner.
pub(crate) fn rewrite(
    plans: &[LogicalPlan],
    purpose: Purpose,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
    enter: impl FnMut(&LogicalPlan, &Scope) -> Result<TreeNodeRecursion>,
    mut update: impl FnMut(LogicalPlan, &Scope, &LogicalPlan) -> Result<Transformed<LogicalPlan>>,
) -> Result<Vec<LogicalPlan>> {
    let reservation = pse_columnar::MemoryConsumer::new("session:scoped-rewrite").register(pool);
    let graph = graph(
        plans,
        purpose,
        cancel,
        |bytes| {
            reservation
                .try_grow(bytes)
                .map_err(|e| pse_columnar::external(crate::EngineError::from(e)))
        },
        enter,
    )?;
    let mut restored = HashMap::<usize, (LogicalPlan, bool)>::new();
    for id in &graph.postorder {
        checkpoint(cancel)?;
        let node = &graph.nodes[*id];
        let mut children = node.children.iter();
        let mut rebuilt = Transformed::no(node.plan.clone());
        if !node.children.is_empty() {
            rebuilt = rebuilt
                .transform_data(|plan| {
                    plan.map_subqueries(|wrapper| {
                        let LogicalPlan::Subquery(mut query) = wrapper else {
                            return Err(invalid("native subquery wrapper absent"));
                        };
                        let (child, changed) = &restored[children
                            .next()
                            .ok_or_else(|| invalid("missing subquery child"))?];
                        if *changed {
                            query.subquery = Arc::new(child.clone());
                        }
                        Ok(Transformed::new_transformed(
                            LogicalPlan::Subquery(query),
                            *changed,
                        ))
                    })
                })?
                .transform_data(|plan| {
                    if purpose == Purpose::Evidence
                        && plan.inputs().is_empty()
                        && super::cache::evidence_input(&plan).is_some()
                    {
                        let (child, _) = &restored[children
                            .next()
                            .ok_or_else(|| invalid("hidden producer child absent"))?];
                        return Ok(Transformed::yes(
                            plan.with_new_exprs(vec![], vec![child.clone()])?,
                        ));
                    }
                    plan.map_children(|_| {
                        let (child, changed) = &restored[children
                            .next()
                            .ok_or_else(|| invalid("missing input child"))?];
                        Ok(Transformed::new_transformed(child.clone(), *changed))
                    })
                })?;
        }
        // DataFusion may reconstruct an extension even when no child reports a
        // change. Give the callback the retained owner in that case so incidental
        // wrapper allocation cannot erase private admission or sharing state.
        let input = if rebuilt.transformed {
            rebuilt.data
        } else {
            node.plan.clone()
        };
        let updated = update(input, &node.scope, &node.plan)?;
        let changed = rebuilt.transformed || updated.transformed;
        restored.insert(
            *id,
            (
                if changed {
                    updated.data
                } else {
                    node.plan.clone()
                },
                changed,
            ),
        );
    }
    graph
        .roots
        .iter()
        .map(|id| {
            restored
                .get(id)
                .map(|(plan, _)| plan.clone())
                .ok_or_else(|| invalid("incomplete scoped rewrite"))
        })
        .collect()
}
pub(crate) fn visit(
    plan: &LogicalPlan,
    purpose: Purpose,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
    f: impl FnMut(&LogicalPlan, &Scope) -> Result<TreeNodeRecursion>,
) -> Result<usize> {
    let reservation = pse_columnar::MemoryConsumer::new("session:scoped-traversal").register(pool);
    visit_many(
        [plan],
        purpose,
        cancel,
        |bytes| {
            reservation
                .try_grow(bytes)
                .map_err(|e| pse_columnar::external(crate::EngineError::from(e)))
        },
        f,
    )
}
pub(crate) fn visit_many<'a>(
    plans: impl IntoIterator<Item = &'a LogicalPlan>,
    purpose: Purpose,
    cancel: &CancellationToken,
    charge: impl FnMut(usize) -> Result<()>,
    f: impl FnMut(&LogicalPlan, &Scope) -> Result<TreeNodeRecursion>,
) -> Result<usize> {
    Ok(graph(plans, purpose, cancel, charge, f)?.nodes.len())
}
#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::logical_expr::{LogicalPlanBuilder, Union};
    #[test]
    fn scoped_reuse_unit_shared_producers_are_bounded_and_cancelable() {
        let mut plan = LogicalPlanBuilder::empty(true).build().unwrap();
        for _ in 0..24 {
            let child = Arc::new(plan);
            plan = LogicalPlan::Union(Union {
                inputs: vec![child.clone(), child.clone()],
                schema: child.schema().clone(),
            });
        }
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let cancel = CancellationToken::new();
        assert_eq!(
            visit(&plan, Purpose::Evidence, &budget, &cancel, |_, _| Ok(
                TreeNodeRecursion::Continue
            ))
            .unwrap(),
            25
        );
        assert_eq!(budget.reserved(), 0);
        cancel.cancel();
        assert!(
            visit(&plan, Purpose::Evidence, &budget, &cancel, |_, _| Ok(
                TreeNodeRecursion::Continue
            ))
            .is_err()
        );
    }
}

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    use datafusion::logical_expr::{LogicalPlanBuilder, Union};
    use pse_schema::model::provider::OperationEffect;

    #[test]
    fn effect_qualification_only_distinguishes_freshness_visits() {
        use datafusion::execution::session_state::{CacheFactory, SessionStateBuilder};
        let state = SessionStateBuilder::new().with_default_features().build();
        let contracted = super::super::cache::NativeCacheFactory
            .create(LogicalPlanBuilder::empty(false).build().unwrap(), &state)
            .unwrap();
        let LogicalPlan::Extension(extension) = &contracted else {
            panic!("contract absent")
        };
        let producer = extension
            .node
            .as_any()
            .downcast_ref::<super::super::contract::ExecutionContract>()
            .unwrap()
            .operation()
            .clone();
        let plan = LogicalPlan::Union(Union {
            schema: contracted.schema().clone(),
            inputs: vec![Arc::new(contracted), Arc::new(producer)],
        });
        let cancel = CancellationToken::new();
        for (purpose, expected) in [(Purpose::Evidence, 4), (Purpose::Freshness, 5)] {
            let graph = graph(
                [&plan],
                purpose,
                &cancel,
                |_| Ok(()),
                |_, _| Ok(TreeNodeRecursion::Continue),
            )
            .unwrap();
            assert_eq!(graph.nodes.len(), expected);
        }
        assert!(
            super::super::freshness::check([&plan], |_| Ok(()), &cancel).unwrap(),
            "a qualified sibling must not authorize the unqualified producer"
        );
    }

    #[test]
    fn closed_producers_do_not_inherit_ancestor_subquery_paths() {
        use datafusion::{
            execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder},
            logical_expr::{Projection, Subquery, lit},
        };
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
        let factory = super::super::EngineFactory::from_builder(
            Arc::new(RuntimeEnv::default()),
            pool.clone(),
            "closed-scope-unit",
            SessionStateBuilder::new().with_default_features(),
        );
        let cancel = CancellationToken::new();
        let session = factory
            .candidate(
                std::collections::BTreeMap::new(),
                Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
                &cancel,
            )
            .unwrap();
        let mut plan = LogicalPlanBuilder::empty(true)
            .project([lit(1i64).alias("n")])
            .unwrap()
            .build()
            .unwrap();
        for _ in 0..18 {
            let shared = Arc::new(session.cache_plan(plan, &cancel).unwrap());
            let expression = || {
                Expr::ScalarSubquery(Subquery {
                    subquery: shared.clone(),
                    outer_ref_columns: vec![],
                    spans: datafusion::common::Spans::default(),
                })
            };
            plan = LogicalPlan::Projection(
                Projection::try_new(
                    vec![(expression() + expression()).alias("n")],
                    Arc::new(LogicalPlanBuilder::empty(true).build().unwrap()),
                )
                .unwrap(),
            );
        }
        let visits = visit(&plan, Purpose::Evidence, &pool, &cancel, |_, _| {
            Ok(TreeNodeRecursion::Continue)
        })
        .unwrap();
        assert!(
            visits < 200,
            "closed producers expanded ancestor subquery paths: {visits}"
        );
        // Context-dependent inputs remain unproved and keep lexical scope; the
        // existing open/correlated producer controls exercise their refusal.
    }

    #[test]
    fn distinct_contract_parents_share_intrinsic_structure() {
        let read = [OperationEffect::Read].into_iter().collect();
        let mut plan = super::super::contract::ExecutionContract::plan(
            LogicalPlanBuilder::empty(true).build().unwrap(),
            None,
            read,
        );
        for _ in 0..18 {
            let parents = [0, 1].map(|_| {
                Arc::new(super::super::contract::ExecutionContract::plan(
                    plan.clone(),
                    None,
                    [OperationEffect::Read].into_iter().collect(),
                ))
            });
            plan = LogicalPlan::Union(Union {
                schema: plan.schema().clone(),
                inputs: parents.into(),
            });
        }
        let cancel = CancellationToken::new();
        let graph = graph(
            [&plan],
            Purpose::Evidence,
            &cancel,
            |_| Ok(()),
            |_, _| Ok(TreeNodeRecursion::Continue),
        )
        .unwrap();
        // Two retained union values per inner level, two contract owners per
        // level, one root union and the initial contract/empty input.
        assert_eq!(graph.nodes.len(), 4 * 18 + 1);
        assert!(!super::super::freshness::check([&plan], |_| Ok(()), &cancel).unwrap());
        let effect = super::super::contract::ExecutionContract::plan(
            plan.clone(),
            None,
            [OperationEffect::Write].into_iter().collect(),
        );
        assert!(super::super::freshness::check([&plan, &effect], |_| Ok(()), &cancel).unwrap());
    }
}
