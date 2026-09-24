// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolate native DML and defer namespace commands once per actual shared owner.
use super::{IsolatedTarget, external, invalid};
use crate::session::{
    EngineSession,
    admission::{NodeIdentity, identity},
};
use datafusion::{
    common::{
        Result,
        tree_node::{Transformed, TreeNode, TreeNodeRecursion, TreeNodeVisitor},
    },
    datasource::{provider_as_source, source_as_provider},
    logical_expr::LogicalPlan,
};
use pse_columnar::CancellationToken;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub(crate) fn isolate(
    plan: &LogicalPlan,
    session: &EngineSession,
    cancel: &CancellationToken,
) -> Result<LogicalPlan> {
    // This root pins every original child while address-based keys are in use.
    let mut visitor = Isolator {
        session,
        cancel,
        nodes: HashMap::new(),
        originals: HashSet::from([identity(plan)]),
        reservation: pse_columnar::MemoryConsumer::new("session:mutation-isolation")
            .register(&session.pool),
    };
    plan.visit_with_subqueries(&mut visitor)?;
    Ok(visitor.child(identity(plan))?.data)
}

struct Isolator<'a> {
    session: &'a EngineSession,
    cancel: &'a CancellationToken,
    nodes: HashMap<NodeIdentity, (LogicalPlan, bool)>,
    originals: HashSet<NodeIdentity>,
    reservation: pse_columnar::MemoryReservation,
}
impl Isolator<'_> {
    fn child(&self, key: NodeIdentity) -> Result<Transformed<LogicalPlan>> {
        self.nodes
            .get(&key)
            .map(|(plan, changed)| Transformed::new_transformed(plan.clone(), *changed))
            .ok_or_else(|| invalid("mutation isolation child was not visited"))
    }
}
impl<'n> TreeNodeVisitor<'n> for Isolator<'_> {
    type Node = LogicalPlan;

    fn f_down(&mut self, node: &'n LogicalPlan) -> Result<TreeNodeRecursion> {
        self.cancel
            .checkpoint()
            .map_err(|error| external(error.into()))?;
        if self.nodes.contains_key(&identity(node)) {
            return Ok(TreeNodeRecursion::Jump);
        }
        self.reservation
            .try_grow(2 * size_of::<LogicalPlan>() + 256)
            .map_err(|error| external(error.into()))?;
        self.originals
            .extend(node.inputs().into_iter().map(identity));
        Ok(TreeNodeRecursion::Continue)
    }

    fn f_up(&mut self, node: &'n LogicalPlan) -> Result<TreeNodeRecursion> {
        let key = identity(node);
        // Native traversal synthesizes temporary Subquery wrappers. Rebuild the
        // real expression-owned wrapper from its retained child instead.
        if !self.originals.contains(&key) || self.nodes.contains_key(&key) {
            return Ok(TreeNodeRecursion::Continue);
        }
        let mut inputs = node.inputs().into_iter().map(identity);
        let rebuilt = node
            .clone()
            .map_subqueries(|plan| {
                let LogicalPlan::Subquery(mut query) = plan else {
                    return Err(invalid("mutation isolation expected native subquery"));
                };
                let child = self.child(identity(query.subquery.as_ref()))?;
                if child.transformed {
                    query.subquery = Arc::new(child.data);
                    Ok(Transformed::yes(LogicalPlan::Subquery(query)))
                } else {
                    Ok(Transformed::no(LogicalPlan::Subquery(query)))
                }
            })?
            .transform_data(|plan| {
                plan.map_children(|_| {
                    self.child(
                        inputs
                            .next()
                            .ok_or_else(|| invalid("mutation isolation input absent"))?,
                    )
                })
            })?;
        let mut changed = rebuilt.transformed;
        let mut output = rebuilt.data;
        if let LogicalPlan::Dml(dml) = &mut output {
            let source = source_as_provider(&dml.target)?;
            let binding = self
                .session
                .bindings
                .iter()
                .find_map(|(_, binding)| Arc::ptr_eq(&binding.provider, &source).then_some(binding))
                .ok_or_else(|| {
                    invalid("DML target is outside the actual bound provider hierarchy")
                })?;
            // Planning still performs no write or private factory invocation.
            dml.target = provider_as_source(Arc::new(IsolatedTarget {
                binding,
                session: self.session.clone(),
            }));
            changed = true;
        }
        let deferred = crate::session::commands::deferred::defer_node(output)?;
        changed |= deferred.transformed;
        self.nodes.insert(
            key,
            (if changed { deferred.data } else { node.clone() }, changed),
        );
        Ok(TreeNodeRecursion::Continue)
    }
}
