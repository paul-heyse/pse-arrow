// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Plan each closed native cache producer once, without executing it. These
//! temporary logical leaves belong only to physical planning; public logical
//! plans retain all producers, requirements and sources.
use super::{Cache, CacheExec, invalid};
use crate::session::execution::NativeExecutionContext;
use datafusion::{
    catalog::Session,
    common::{
        Result,
        tree_node::{Transformed, TreeNodeRecursion, TreeNodeVisitor},
    },
    execution::session_state::SessionState,
    logical_expr::{Extension, LogicalPlan, UserDefinedLogicalNode},
    physical_plan::ExecutionPlan,
    physical_planner::PhysicalPlanner,
};
use pse_ids::Reservation;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

type Prepared = HashMap<usize, (Arc<dyn UserDefinedLogicalNode>, LogicalPlan)>;

pub(in crate::session) async fn prepare(
    plan: LogicalPlan,
    session: &dyn Session,
    planner: &dyn PhysicalPlanner,
) -> Result<LogicalPlan> {
    let Some(services) = session
        .as_any()
        .downcast_ref::<SessionState>()
        .and_then(|state| state.config().get_extension::<NativeExecutionContext>())
    else {
        return Ok(plan);
    };
    let mut inventory = Inventory {
        seen: HashSet::new(),
        producers: Vec::new(),
        reservation: services.reserver().open("session:cache-producer-inventory"),
    };
    plan.visit_with_subqueries(&mut inventory)?;
    let mut prepared = Prepared::new();
    for owner in inventory.producers {
        services.cancellation().checkpoint().map_err(|error| {
            datafusion::common::DataFusionError::External(Box::new(crate::CatalogError::from(
                error,
            )))
        })?;
        let cache = owner
            .as_any()
            .downcast_ref::<Cache>()
            .ok_or_else(|| invalid("cache inventory changed type"))?;
        let input = replace(cache.input.clone(), &prepared)?;
        let physical = if super::logical::closed(&input)? {
            if let Some(state) = session.as_any().downcast_ref::<SessionState>() {
                crate::cache_service::metrics::record(state, |metrics| &metrics.physical_plans);
            }
            let input = planner.create_physical_plan(&input, session).await?;
            if !matches!(
                input.properties().boundedness,
                datafusion::physical_plan::execution_plan::Boundedness::Bounded
            ) {
                return Err(invalid("materialized cache needs a bounded input"));
            }
            let mut producer = CacheExec::new(input, services.caches.cell(&cache.identity)?);
            producer.planned_boundary = true;
            let producer: Arc<dyn ExecutionPlan> = Arc::new(producer);
            super::record_physical(&producer, session)?;
            Some(producer)
        } else {
            // Correlated/worktable scopes must remain inside their native query.
            None
        };
        let node = LogicalPlan::Extension(Extension {
            node: Arc::new(Cache {
                input,
                prepared: physical,
                admission: None,
                ..cache.clone()
            }),
        });
        prepared.insert(Arc::as_ptr(&owner).cast::<()>() as usize, (owner, node));
    }
    replace(plan, &prepared)
}

struct Inventory {
    seen: HashSet<crate::session::admission::NodeIdentity>,
    producers: Vec<Arc<dyn UserDefinedLogicalNode>>,
    reservation: Box<dyn Reservation>,
}
impl<'n> TreeNodeVisitor<'n> for Inventory {
    type Node = LogicalPlan;
    fn f_down(&mut self, node: &'n LogicalPlan) -> Result<TreeNodeRecursion> {
        if !matches!(node, LogicalPlan::Subquery(_)) {
            let key = crate::session::admission::identity(node);
            if self.seen.contains(&key) {
                return Ok(TreeNodeRecursion::Jump);
            }
            self.reservation
                .try_grow(size_of::<LogicalPlan>() + 192)
                .map_err(|error| {
                    datafusion::common::DataFusionError::External(Box::new(
                        crate::CatalogError::from(error),
                    ))
                })?;
            self.seen.insert(key);
        }
        Ok(TreeNodeRecursion::Continue)
    }
    fn f_up(&mut self, node: &'n LogicalPlan) -> Result<TreeNodeRecursion> {
        if let LogicalPlan::Extension(extension) = node
            && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
            && cache.prepared.is_none()
            && !self
                .producers
                .iter()
                .any(|owner| Arc::ptr_eq(owner, &extension.node))
        {
            self.producers.push(Arc::clone(&extension.node));
        }
        Ok(TreeNodeRecursion::Continue)
    }
}

fn replace(plan: LogicalPlan, prepared: &Prepared) -> Result<LogicalPlan> {
    Ok(plan
        .transform_down_with_subqueries(|node| {
            if let LogicalPlan::Extension(extension) = &node
                && let Some((_, replacement)) =
                    prepared.get(&(Arc::as_ptr(&extension.node).cast::<()>() as usize))
            {
                return Ok(Transformed::new(
                    replacement.clone(),
                    true,
                    TreeNodeRecursion::Jump,
                ));
            }
            Ok(Transformed::no(node))
        })?
        .data)
}
