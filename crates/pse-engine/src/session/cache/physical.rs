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
        tree_node::{Transformed, TreeNodeRecursion},
    },
    execution::session_state::SessionState,
    logical_expr::{Extension, LogicalPlan, UserDefinedLogicalNode},
    physical_plan::ExecutionPlan,
    physical_planner::PhysicalPlanner,
};
use std::{collections::HashMap, sync::Arc};

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
    let reservation = pse_columnar::MemoryConsumer::new("session:cache-producer-inventory")
        .register(services.pool());
    let graph = crate::session::traversal::graph(
        [&plan],
        crate::session::traversal::Purpose::Evidence,
        services.cancellation(),
        |bytes| {
            reservation
                .try_grow(bytes)
                .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))
        },
        |_, _| Ok(TreeNodeRecursion::Continue),
    )?;
    let mut prepared = Prepared::new();
    for id in &graph.postorder {
        let LogicalPlan::Extension(extension) = &graph.nodes[*id].plan else {
            continue;
        };
        let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
            continue;
        };
        let owner = Arc::clone(&extension.node);
        if cache.prepared.is_some()
            || prepared.contains_key(&(Arc::as_ptr(&owner).cast::<()>() as usize))
        {
            continue;
        }
        services
            .cancellation()
            .checkpoint()
            .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))?;
        let cache = owner
            .as_any()
            .downcast_ref::<Cache>()
            .ok_or_else(|| invalid("cache inventory changed type"))?;
        let input = replace(cache.input.clone(), &prepared)?;
        let retained = services
            .caches
            .0
            .lock()
            .map_err(|_| invalid("cache owner lock poisoned"))?
            .get(&(Arc::as_ptr(&cache.identity) as usize))
            .and_then(|(_, _, physical)| physical.as_ref())
            .and_then(std::sync::Weak::upgrade);
        let physical = if retained.is_some() {
            retained
        } else if super::logical::closed(&input, services.pool(), services.cancellation())? {
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
            producer.model = super::model_binding(cache, session);
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
