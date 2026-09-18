// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Invocation-local native producers for unchanged value sides of witness joins.
use super::{Registry, RuleSpec, SupportPlan, internal};
use crate::RuleError;
use datafusion_expr::LogicalPlan;
use pse_catalog::{CatalogError, session::SnapshotSession};
use pse_ids::{CancellationToken, Reservation};
use std::collections::HashMap;

type Key = (usize, Option<&'static str>);
pub(super) type Traces = HashMap<Key, Vec<SupportPlan>>;

fn key(plan: &LogicalPlan, port: Option<&'static str>) -> Key {
    (std::ptr::from_ref(plan) as usize, port)
}

pub(super) fn child_port(
    plan: &LogicalPlan,
    rule: &RuleSpec,
    port: Option<&'static str>,
) -> Option<&'static str> {
    if let LogicalPlan::SubqueryAlias(alias) = plan {
        rule.inputs
            .iter()
            .find(|input| input.port == alias.alias.table())
            .map(|input| input.port)
            .or(port)
    } else {
        port
    }
}

pub(super) fn traced(
    plan: &LogicalPlan,
    port: Option<&'static str>,
    traces: &Traces,
) -> Result<Vec<SupportPlan>, RuleError> {
    traces
        .get(&key(plan, port))
        .cloned()
        .ok_or_else(|| internal("native witness child was not prepared"))
}

pub(super) fn trace(
    plan: &LogicalPlan,
    rule: &RuleSpec,
    registry: &Registry,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<Vec<SupportPlan>, RuleError> {
    let mut shared = Shared::new(session, cancel);
    let mut traces = Traces::new();
    let mut pending = vec![(plan, None, false)];
    while let Some((node, port, ready)) = pending.pop() {
        cancel.checkpoint().map_err(CatalogError::from)?;
        let key = key(node, port);
        if traces.contains_key(&key) {
            continue;
        }
        if ready {
            let result = super::trace_node(node, rule, registry, port, &mut shared, &traces)?;
            shared.charge(result.len().saturating_mul(size_of::<SupportPlan>() + 128))?;
            traces.insert(key, result);
        } else {
            let children = node.inputs();
            shared.charge((children.len() + 1).saturating_mul(128))?;
            pending.push((node, port, true));
            let next_port = child_port(node, rule, port);
            pending.extend(
                children
                    .into_iter()
                    .rev()
                    .map(|child| (child, next_port, false)),
            );
        }
    }
    traces
        .remove(&key(plan, None))
        .ok_or_else(|| internal("native witness root was not prepared"))
}

pub(super) struct Shared<'a> {
    session: &'a SnapshotSession,
    cancel: &'a CancellationToken,
    originals: HashMap<usize, LogicalPlan>,
    reservation: Box<dyn Reservation>,
}

impl<'a> Shared<'a> {
    pub(super) fn new(session: &'a SnapshotSession, cancel: &'a CancellationToken) -> Self {
        Self {
            session,
            cancel,
            originals: HashMap::new(),
            reservation: session.reserver().open("rules:shared-support-values"),
        }
    }

    pub(super) fn original(&mut self, plan: &LogicalPlan) -> Result<LogicalPlan, RuleError> {
        self.cancel.checkpoint().map_err(CatalogError::from)?;
        // The immutable query borrowed by trace owns every key until this map drops.
        // Leaves already retain their native provider; materializing them adds no
        // shared operator work and would inhibit useful native scan pushdown.
        if plan.inputs().is_empty() {
            return Ok(plan.clone());
        }
        let key = std::ptr::from_ref(plan) as usize;
        if let Some(original) = self.originals.get(&key) {
            return Ok(original.clone());
        }
        self.reservation
            .try_grow(size_of::<LogicalPlan>() + 128)
            .map_err(CatalogError::from)?;
        let original = self.session.cache_plan(plan.clone(), self.cancel)?;
        self.originals.insert(key, original.clone());
        Ok(original)
    }

    pub(super) fn charge(&mut self, bytes: usize) -> Result<(), RuleError> {
        self.cancel.checkpoint().map_err(CatalogError::from)?;
        self.reservation
            .try_grow(bytes)
            .map_err(CatalogError::from)?;
        Ok(())
    }
}
