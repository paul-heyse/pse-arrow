// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native optimizer boundaries for shared, closed cache producers. These are
//! temporary planning nodes, never new data authorities or executable fallbacks.
use super::Cache;
use datafusion::{
    common::{
        Result,
        tree_node::{Transformed, TreeNodeRecursion},
    },
    datasource::source_as_provider,
    logical_expr::{Expr, Extension, LogicalPlan, UserDefinedLogicalNode},
};

use pse_columnar::{CancellationToken, MemoryPool};
use std::{collections::HashMap, sync::Arc};

pub(crate) fn stage(
    plan: LogicalPlan,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
    optimize: &mut impl FnMut(LogicalPlan, bool) -> Result<LogicalPlan>,
) -> Result<LogicalPlan> {
    Scope::new(pool, cancel).stage(plan, optimize)
}

type Entry = (Arc<dyn UserDefinedLogicalNode>, LogicalPlan);

/// SQL name resolution needs a producer's output schema, not another traversal
/// and rewrite of every closed producer behind an already admitted view. Keep the
/// exact original owner and restore it before returning the bound native query.
pub(crate) fn for_query_binding(
    plan: LogicalPlan,
    registry: &pse_schema::Registry,
    tables: &[Arc<dyn datafusion::catalog::TableProvider>],
    reservation: &mut pse_columnar::MemoryReservation,
    cancel: &CancellationToken,
) -> Result<LogicalPlan> {
    Ok(plan
        .transform_down_with_subqueries(|node| {
            cancel
                .checkpoint()
                .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))?;
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
                .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))?;
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

pub struct Scope<'a> {
    owner: Option<super::super::reuse::Witness>,
    entries: HashMap<(usize, bool), Entry>,
    reservation: pse_columnar::MemoryReservation,
    cancel: &'a CancellationToken,
    pool: &'a Arc<dyn MemoryPool>,
}
impl<'a> Scope<'a> {
    pub fn new(pool: &'a Arc<dyn MemoryPool>, cancel: &'a CancellationToken) -> Self {
        Self {
            owner: None,
            entries: HashMap::new(),
            reservation: pse_columnar::MemoryConsumer::new("session:cache-producer-planning")
                .register(pool),
            cancel,
            pool,
        }
    }

    pub(crate) fn bind_owner(&mut self, session: &super::super::EngineSession) -> Result<()> {
        if let Some(owner) = &self.owner {
            if !owner.matches(session) {
                return Err(super::invalid(
                    "shared preparation scope changed its actual semantic owners",
                ));
            }
        } else {
            let owner =
                super::super::reuse::Witness::capture(session).map_err(pse_columnar::external)?;
            self.reservation
                .try_grow(owner.retained_bytes())
                .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))?;
            self.owner = Some(owner);
        }
        Ok(())
    }

    pub(crate) fn stage(
        &mut self,
        plan: LogicalPlan,
        optimize: &mut impl FnMut(LogicalPlan, bool) -> Result<LogicalPlan>,
    ) -> Result<LogicalPlan> {
        super::super::traversal::visit(
            &plan,
            super::super::traversal::Purpose::Rewrite,
            self.pool,
            self.cancel,
            |_, _| Ok(TreeNodeRecursion::Continue),
        )?;
        let plan = super::super::contract::protect(plan, self.pool, self.cancel)?;
        super::super::traversal::rewrite(
            &[plan],
            super::super::traversal::Purpose::Rewrite,
            self.pool,
            self.cancel,
            |_, _| Ok(TreeNodeRecursion::Continue),
            |node, _scope, original| {
                self.cancel
                    .checkpoint()
                    .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))?;
                let LogicalPlan::Extension(extension) = &node else {
                    return Ok(Transformed::no(node));
                };
                let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
                    return Ok(Transformed::no(node));
                };
                // Only closed producers enter this table: their lexical context is
                // intrinsic to the retained input. Owner binding scopes this table
                // to the actual semantic assembly; required work is a separate key.
                let LogicalPlan::Extension(original) = original else {
                    return Err(super::invalid("cache owner absent"));
                };
                let original_cache = original
                    .node
                    .as_any()
                    .downcast_ref::<Cache>()
                    .ok_or_else(|| super::invalid("cache owner changed type"))?;
                let key = (
                    Arc::as_ptr(&original_cache.identity) as usize,
                    cache.required,
                );
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
                let input = cache.input.clone();
                // Correlated/worktable inputs must stay in their enclosing native scope.
                if !closed(&input, self.pool, self.cancel)? {
                    return Ok(Transformed::no(node));
                }
                let input = optimize(input, cache.required)?;
                self.reservation
                    .try_grow(size_of::<Entry>() + 128)
                    .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))?;
                let staged = LogicalPlan::Extension(Extension {
                    node: Arc::new(Cache {
                        input,
                        optimizer_leaf: true,
                        identity: Arc::clone(&original_cache.identity),
                        admission: None,
                        repeatable_input: false,
                        ..cache.clone()
                    }),
                });
                self.entries
                    .insert(key, (Arc::clone(&extension.node), staged.clone()));
                Ok(Transformed::new(staged, true, TreeNodeRecursion::Jump))
            },
        )?
        .pop()
        .ok_or_else(|| super::invalid("staged native root absent"))
    }
}

pub(in crate::session) fn closed(
    plan: &LogicalPlan,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<bool> {
    let mut closed = true;
    crate::session::traversal::visit(
        plan,
        crate::session::traversal::Purpose::Evidence,
        pool,
        cancel,
        |node, scope| {
            if let LogicalPlan::Extension(extension) = node
                && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                && cache.admission.is_some()
            {
                // A structural proof is created only for a closed owner and is
                // discarded whenever that owner's native inputs change.
                return Ok(TreeNodeRecursion::Jump);
            }
            closed &= !scope.unbound;
            if let LogicalPlan::TableScan(scan) = node
                && let Some(work) = source_as_provider(&scan.source)?
                    .downcast_ref::<datafusion_catalog::cte_worktable::CteWorkTable>(
                )
            {
                closed &= scope.worktables.iter().any(|(name, _)| name == work.name());
            }
            node.apply_expressions(|expr| {
                crate::session::traversal::expression(expr, |expr| {
                    if matches!(expr, Expr::OuterReferenceColumn(..)) {
                        closed &= scope.outer.contains(expr);
                    }
                    Ok(TreeNodeRecursion::Continue)
                })
            })?;
            Ok(if closed {
                TreeNodeRecursion::Continue
            } else {
                TreeNodeRecursion::Stop
            })
        },
    )?;
    Ok(closed)
}

/// Restore complete native children before policy, admission or public observation.
pub(crate) fn expand(
    plan: LogicalPlan,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<LogicalPlan> {
    let mut entries = HashMap::<(usize, super::super::traversal::Scope), Entry>::new();
    let memory =
        pse_columnar::MemoryConsumer::new("session:visible-producer-owners").register(pool);
    super::super::traversal::rewrite(
        &[plan],
        super::super::traversal::Purpose::Evidence,
        pool,
        cancel,
        |node, _| {
            Ok(
                if let LogicalPlan::Extension(extension) = node
                    && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                    && !cache.optimizer_leaf
                    && cache.admission.is_some()
                {
                    TreeNodeRecursion::Jump
                } else {
                    TreeNodeRecursion::Continue
                },
            )
        },
        |node, scope, original| {
            let LogicalPlan::Extension(extension) = &node else {
                return Ok(Transformed::no(node));
            };
            let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
                return Ok(Transformed::no(node));
            };
            if !cache.optimizer_leaf && cache.admission.is_some() {
                return Ok(Transformed::no(node));
            }
            let LogicalPlan::Extension(original) = original else {
                return Err(super::invalid("original cache owner absent"));
            };
            let prior = original
                .node
                .as_any()
                .downcast_ref::<Cache>()
                .ok_or_else(|| super::invalid("original cache type changed"))?;
            let key = (
                Arc::as_ptr(&prior.identity) as usize,
                if closed(&prior.input, pool, cancel)? {
                    super::super::traversal::Scope::default()
                } else {
                    scope.clone()
                },
            );
            if let Some((owner, restored)) = entries.get(&key) {
                if owner.schema() != original.node.schema() {
                    return Err(super::invalid("one visible cache producer changed schema"));
                }
                return Ok(Transformed::yes(restored.clone()));
            }
            memory
                .try_grow(size_of::<Entry>() + 256)
                .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))?;
            let restored = LogicalPlan::Extension(Extension {
                node: Arc::new(Cache {
                    identity: prior.identity.clone(),
                    optimizer_leaf: false,
                    prepared: None,
                    admission: None,
                    repeatable_input: false,
                    ..cache.clone()
                }),
            });
            entries.insert(key, (original.node.clone(), restored.clone()));
            Ok(Transformed::yes(restored))
        },
    )?
    .pop()
    .ok_or_else(|| super::invalid("expanded native root absent"))
}

impl std::fmt::Debug for Scope<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::common::tree_node::TreeNode;
    use datafusion::{
        execution::context::SessionContext,
        logical_expr::{LogicalPlanBuilder, Union, lit},
    };

    #[tokio::test]
    async fn correlated_subqueries_are_closed_only_with_their_actual_parent() {
        let context = SessionContext::new();
        let plan = context.sql("SELECT p.n FROM (VALUES (1), (2)) p(n) WHERE EXISTS (SELECT 1 FROM (VALUES (1)) q(n) WHERE q.n = p.n)").await.unwrap().into_unoptimized_plan();
        assert!(
            closed(
                &plan,
                &{
                    let pool: Arc<dyn MemoryPool> =
                        Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
                    pool
                },
                &CancellationToken::new()
            )
            .unwrap()
        );
        let mut checked = 0;
        plan.apply(|node| {
            node.apply_expressions(|expression| {
                expression.apply(|expression| {
                    if let Expr::Exists(exists) = expression {
                        assert!(
                            !closed(
                                &exists.subquery.subquery,
                                &{
                                    let pool: Arc<dyn MemoryPool> =
                                        Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
                                    pool
                                },
                                &CancellationToken::new()
                            )
                            .unwrap()
                        );
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
                required: false,
                admission: None,
                repeatable_input: false,
                binding: None,
                retention: None,
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
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let mut calls = 0;
        let staged = stage(
            input.clone(),
            &budget,
            &CancellationToken::new(),
            &mut |input, _| {
                calls += 1;
                state.optimize(&input)
            },
        )
        .unwrap();
        assert_eq!(
            calls, 6,
            "shared producer identity survives native node reconstruction"
        );
        let expanded = expand(staged, &budget, &CancellationToken::new()).unwrap();
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
        let restored = expand(expanded, &budget, &CancellationToken::new()).unwrap();
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
