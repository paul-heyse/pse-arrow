// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native optimizer boundaries for shared, closed cache producers. These are
//! temporary planning nodes, never new data authorities or executable fallbacks.
use super::Cache;
use datafusion::{
    common::{
        Result,
        tree_node::{Transformed, TreeNode, TreeNodeRecursion},
    },
    datasource::source_as_provider,
    logical_expr::{Expr, Extension, LogicalPlan, UserDefinedLogicalNode},
};
use pse_ids::{CancellationToken, MemoryReserver, Reservation};
use std::{collections::HashMap, sync::Arc};

pub(crate) fn stage(
    plan: LogicalPlan,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
    optimize: &mut impl FnMut(LogicalPlan) -> Result<LogicalPlan>,
) -> Result<LogicalPlan> {
    Scope::new(reserver, cancel).stage(plan, optimize)
}

type Entry = (Arc<dyn UserDefinedLogicalNode>, LogicalPlan);

/// SQL name resolution needs a producer's output schema, not another traversal
/// and rewrite of every closed producer behind an already admitted view. Keep the
/// exact original owner and restore it before returning the bound native query.
pub(crate) fn for_query_binding(
    plan: LogicalPlan,
    registry: &pse_schema::Registry,
    tables: &[Arc<dyn datafusion::catalog::TableProvider>],
    reservation: &mut dyn Reservation,
    cancel: &CancellationToken,
) -> Result<LogicalPlan> {
    Ok(plan
        .transform_down_with_subqueries(|node| {
            cancel.checkpoint().map_err(|error| {
                datafusion::common::DataFusionError::External(Box::new(crate::CatalogError::from(
                    error,
                )))
            })?;
            if !super::admitted(&node, registry, tables) {
                return Ok(Transformed::no(node));
            }
            let LogicalPlan::Extension(extension) = &node else {
                return Ok(Transformed::no(node));
            };
            let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
                return Ok(Transformed::no(node));
            };
            reservation
                .try_grow(size_of::<Cache>() + 128)
                .map_err(|error| {
                    datafusion::common::DataFusionError::External(Box::new(
                        crate::CatalogError::from(error),
                    ))
                })?;
            Ok(Transformed::new(
                LogicalPlan::Extension(Extension {
                    node: Arc::new(Cache {
                        optimizer_leaf: true,
                        binding: Some(Arc::clone(&extension.node)),
                        ..cache.clone()
                    }),
                }),
                true,
                TreeNodeRecursion::Jump,
            ))
        })?
        .data)
}

pub(crate) fn restore_query_bindings(plan: LogicalPlan) -> Result<LogicalPlan> {
    Ok(plan
        .transform_down_with_subqueries(|node| {
            if let LogicalPlan::Extension(extension) = &node
                && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                && let Some(original) = &cache.binding
            {
                return Ok(Transformed::new(
                    LogicalPlan::Extension(Extension {
                        node: Arc::clone(original),
                    }),
                    true,
                    TreeNodeRecursion::Jump,
                ));
            }
            Ok(Transformed::no(node))
        })?
        .data)
}

pub(crate) struct Scope<'a> {
    entries: HashMap<usize, Entry>,
    reservation: Box<dyn Reservation>,
    cancel: &'a CancellationToken,
}
impl<'a> Scope<'a> {
    pub(crate) fn new(reserver: &dyn MemoryReserver, cancel: &'a CancellationToken) -> Self {
        Self {
            entries: HashMap::new(),
            reservation: reserver.open("session:cache-producer-planning"),
            cancel,
        }
    }

    pub(crate) fn stage(
        &mut self,
        plan: LogicalPlan,
        optimize: &mut impl FnMut(LogicalPlan) -> Result<LogicalPlan>,
    ) -> Result<LogicalPlan> {
        Ok(plan
            .transform_down_with_subqueries(|node| {
                self.cancel.checkpoint().map_err(|error| {
                    datafusion::common::DataFusionError::External(Box::new(
                        crate::CatalogError::from(error),
                    ))
                })?;
                let LogicalPlan::Extension(extension) = &node else {
                    return Ok(Transformed::no(node));
                };
                let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
                    return Ok(Transformed::no(node));
                };
                let key = Arc::as_ptr(&cache.identity) as usize;
                if let Some((original, staged)) = self.entries.get(&key) {
                    // Compare two references to the original producer. Native
                    // optimization may legitimately refine its output nullability.
                    if original.schema() != node.schema() {
                        return Err(super::invalid(
                            "one cache producer has conflicting native schemas",
                        ));
                    }
                    return Ok(Transformed::new(
                        staged.clone(),
                        true,
                        TreeNodeRecursion::Jump,
                    ));
                }
                let input = self.stage(cache.input.clone(), optimize)?;
                // Correlated/worktable inputs must stay in their enclosing native scope.
                if !closed(&input)? {
                    return Ok(Transformed::no(node));
                }
                let input = optimize(input)?;
                self.reservation
                    .try_grow(size_of::<Entry>() + 128)
                    .map_err(|error| {
                        datafusion::common::DataFusionError::External(Box::new(
                            crate::CatalogError::from(error),
                        ))
                    })?;
                let staged = LogicalPlan::Extension(Extension {
                    node: Arc::new(Cache {
                        input,
                        optimizer_leaf: true,
                        admission: None,
                        ..cache.clone()
                    }),
                });
                self.entries
                    .insert(key, (Arc::clone(&extension.node), staged.clone()));
                Ok(Transformed::new(staged, true, TreeNodeRecursion::Jump))
            })?
            .data)
    }
}

pub(super) fn closed(plan: &LogicalPlan) -> Result<bool> {
    closed_in(plan, &mut Vec::new(), &[])
}

fn closed_in(plan: &LogicalPlan, scope: &mut Vec<String>, outer: &[Expr]) -> Result<bool> {
    let mut closed = true;
    let mut visited = std::collections::HashSet::new();
    plan.apply(|node| {
        // A structural record is issued only for a closed producer and is
        // discarded whenever its input is reconstructed. Closedness is intrinsic
        // to that immutable input, independent of this caller's outer bindings.
        if let LogicalPlan::Extension(extension) = node
            && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
            && cache.admission.is_some()
        {
            return Ok(TreeNodeRecursion::Jump);
        }
        if !matches!(node, LogicalPlan::Subquery(_))
            && !visited.insert(super::super::admission::identity(node))
        {
            return Ok(TreeNodeRecursion::Jump);
        }
        if let LogicalPlan::RecursiveQuery(query) = node {
            closed &= closed_in(&query.static_term, scope, outer)?;
            scope.push(query.name.clone());
            closed &= closed_in(&query.recursive_term, scope, outer)?;
            scope.pop();
            return Ok(if closed {
                TreeNodeRecursion::Jump
            } else {
                TreeNodeRecursion::Stop
            });
        }
        if let LogicalPlan::TableScan(scan) = node
            && let Some(work) = source_as_provider(&scan.source)?
                .downcast_ref::<datafusion_catalog::cte_worktable::CteWorkTable>()
        {
            closed &= scope.iter().any(|name| name == work.name());
        }
        node.apply_expressions(|expression| {
            expression.apply(|expression| {
                closed &= match expression {
                    Expr::OuterReferenceColumn(..) => outer.contains(expression),
                    Expr::Exists(exists) => query_closed(&exists.subquery, node, scope, outer)?,
                    Expr::InSubquery(query) => query_closed(&query.subquery, node, scope, outer)?,
                    Expr::ScalarSubquery(query) => query_closed(query, node, scope, outer)?,
                    _ => true,
                };
                Ok(if closed {
                    TreeNodeRecursion::Continue
                } else {
                    TreeNodeRecursion::Stop
                })
            })?;
            Ok(TreeNodeRecursion::Continue)
        })?;
        Ok(if closed {
            TreeNodeRecursion::Continue
        } else {
            TreeNodeRecursion::Stop
        })
    })?;
    Ok(closed)
}

fn query_closed(
    query: &datafusion::logical_expr::Subquery,
    parent: &LogicalPlan,
    scope: &mut Vec<String>,
    outer: &[Expr],
) -> Result<bool> {
    let owned = query.outer_ref_columns.iter().all(|reference| {
        let Expr::OuterReferenceColumn(_, column) = reference else {
            return false;
        };
        outer.contains(reference)
            || parent.schema().index_of_column(column).is_ok()
            || parent
                .inputs()
                .iter()
                .any(|input| input.schema().index_of_column(column).is_ok())
    });
    Ok(owned && closed_in(&query.subquery, scope, &query.outer_ref_columns)?)
}

/// Restore complete native children before policy, admission or public observation.
pub(crate) fn expand(plan: LogicalPlan) -> Result<LogicalPlan> {
    fn expand_in(plan: LogicalPlan, entries: &mut HashMap<usize, Entry>) -> Result<LogicalPlan> {
        Ok(plan
            .transform_down_with_subqueries(|node| {
                let LogicalPlan::Extension(extension) = &node else {
                    return Ok(Transformed::no(node));
                };
                let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
                    return Ok(Transformed::no(node));
                };
                let key = Arc::as_ptr(&cache.identity) as usize;
                if let Some((_, expanded)) = entries.get(&key) {
                    return Ok(Transformed::new(
                        expanded.clone(),
                        true,
                        TreeNodeRecursion::Jump,
                    ));
                }
                // A sealed original producer has no temporary planning leaves.
                // Keep its exact structural proof and owner. Other unexpanded
                // producers still need recursive restoration, once per identity.
                if !cache.optimizer_leaf && cache.admission.is_some() {
                    entries.insert(key, (Arc::clone(&extension.node), node.clone()));
                    return Ok(Transformed::new(node, false, TreeNodeRecursion::Jump));
                }
                let input = expand_in(cache.input.clone(), entries)?;
                let expanded = LogicalPlan::Extension(Extension {
                    node: Arc::new(Cache {
                        input,
                        optimizer_leaf: false,
                        admission: None,
                        ..cache.clone()
                    }),
                });
                entries.insert(key, (Arc::clone(&extension.node), expanded.clone()));
                Ok(Transformed::new(expanded, true, TreeNodeRecursion::Jump))
            })?
            .data)
    }
    expand_in(plan, &mut HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        execution::context::SessionContext,
        logical_expr::{LogicalPlanBuilder, Union, lit},
    };

    #[tokio::test]
    async fn correlated_subqueries_are_closed_only_with_their_actual_parent() {
        let context = SessionContext::new();
        let plan = context.sql("SELECT p.n FROM (VALUES (1), (2)) p(n) WHERE EXISTS (SELECT 1 FROM (VALUES (1)) q(n) WHERE q.n = p.n)").await.unwrap().into_unoptimized_plan();
        assert!(closed(&plan).unwrap());
        let mut checked = 0;
        plan.apply(|node| {
            node.apply_expressions(|expression| {
                expression.apply(|expression| {
                    if let Expr::Exists(exists) = expression {
                        assert!(!closed(&exists.subquery.subquery).unwrap());
                        checked += 1;
                    }
                    Ok(TreeNodeRecursion::Continue)
                })?;
                Ok(TreeNodeRecursion::Continue)
            })?;
            Ok(TreeNodeRecursion::Continue)
        })
        .unwrap();
        assert_eq!(checked, 1);
    }

    #[test]
    fn cloned_producers_are_optimized_once_and_all_native_children_are_restored() {
        let state = SessionContext::new().state();
        let mut input = LogicalPlanBuilder::empty(true)
            .project([lit(7_i64).alias("n")])
            .unwrap()
            .build()
            .unwrap();
        for _ in 0..6 {
            let producer = Cache {
                input,
                identity: Arc::new(()),
                prepared: None,
                optimizer_leaf: false,
                admission: None,
                binding: None,
            };
            let schema = Arc::clone(producer.input.schema());
            input = LogicalPlan::Union(Union {
                inputs: (0..2)
                    .map(|_| {
                        Arc::new(LogicalPlan::Extension(Extension {
                            node: Arc::new(producer.clone()),
                        }))
                    })
                    .collect(),
                schema,
            });
        }
        let budget = pse_ids::FixedBudget::new(1 << 20);
        let mut calls = 0;
        let staged = stage(
            input.clone(),
            budget.as_ref(),
            &CancellationToken::new(),
            &mut |input| {
                calls += 1;
                state.optimize(&input)
            },
        )
        .unwrap();
        assert_eq!(
            calls, 6,
            "shared producer identity survives native node reconstruction"
        );
        let expanded = expand(staged).unwrap();
        assert_eq!(expanded.schema(), input.schema());
        let mut identities = std::collections::HashSet::new();
        expanded
            .apply_with_subqueries(|node| {
                if let LogicalPlan::Extension(extension) = node
                    && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                {
                    assert!(!cache.optimizer_leaf);
                    assert_eq!(extension.node.inputs().len(), 1);
                    if !identities.insert(Arc::as_ptr(&cache.identity) as usize) {
                        return Ok(TreeNodeRecursion::Jump);
                    }
                }
                Ok(TreeNodeRecursion::Continue)
            })
            .unwrap();
        assert_eq!(identities.len(), 6);
        // A second expansion must also treat the restored plan as a DAG. Every
        // occurrence of an original producer keeps one native owner, even after
        // all temporary optimizer boundaries have already been removed.
        let restored = expand(expanded).unwrap();
        let mut owners = HashMap::new();
        restored
            .apply_with_subqueries(|node| {
                if let LogicalPlan::Extension(extension) = node
                    && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                {
                    assert!(!cache.optimizer_leaf);
                    let key = Arc::as_ptr(&cache.identity) as usize;
                    if let Some(owner) = owners.insert(key, Arc::clone(&extension.node)) {
                        assert!(Arc::ptr_eq(&owner, &extension.node));
                        return Ok(TreeNodeRecursion::Jump);
                    }
                }
                Ok(TreeNodeRecursion::Continue)
            })
            .unwrap();
        assert_eq!(owners.len(), 6);
    }
}
