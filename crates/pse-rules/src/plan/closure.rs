// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite nested recursive programs over owned native intermediate computations.
//! Support is a flat payload/source-key relation, never a derivation tree.
mod shared;

use super::{Compiler, NativeBindings, Planned, PortBinding, lower::compatible, trace::Trace};
use crate::{
    RuleError,
    errmap::{engine, internal},
    strata::relational::{difference, union},
};
use datafusion_expr::{LogicalPlan, LogicalPlanBuilder};
use pse_catalog::{
    BoxFut,
    session::{PlanObservation, SnapshotSession},
};
use pse_ids::CancellationToken;
use pse_schema::{
    Registry,
    model::{DepthBound, RelationKey, RulePlan, RuleSpec},
};
use std::collections::BTreeMap;

type Binders = Vec<(String, Planned)>;
type TraceBinders = Vec<(String, Vec<Trace>)>;
type TraceKey = (RelationKey, &'static str, bool);

pub(crate) struct RoundScope {
    pub(crate) max_rounds: u32,
    pub(crate) ordinal: u32,
    /// A read of an active head must remain visible to input-delta substitution.
    pub(crate) evolving: bool,
}

pub(crate) async fn prepare(
    rule: &RuleSpec,
    ports: &PortBinding,
    session: &SnapshotSession,
    registry: &Registry,
    scope: RoundScope,
    cancel: &CancellationToken,
) -> Result<(NativeBindings, SnapshotSession, Vec<PlanObservation>), RuleError> {
    let mut closure = Closure {
        rule,
        ports,
        registry,
        cancel,
        max_rounds: scope.max_rounds,
        prefix: format!("__pse_nested:{}:{}", rule.id, scope.ordinal),
        counter: 0,
        session: session.clone(),
        native: NativeBindings::default(),
        plans: vec![],
    };
    closure.expand(&rule.plan, &[], &[]).await?;
    if !scope.evolving {
        shared::prepare(&mut closure, &rule.plan).await?;
    }
    Ok((closure.native, closure.session, closure.plans))
}

struct Closure<'a> {
    rule: &'a RuleSpec,
    ports: &'a PortBinding,
    registry: &'a Registry,
    cancel: &'a CancellationToken,
    max_rounds: u32,
    prefix: String,
    counter: usize,
    session: SnapshotSession,
    native: NativeBindings,
    plans: Vec<PlanObservation>,
}

impl Closure<'_> {
    fn expand<'a>(
        &'a mut self,
        source: &'a RulePlan,
        binders: &'a [(String, Planned)],
        traces: &'a [(String, Vec<Trace>)],
    ) -> BoxFut<'a, Result<(), RuleError>> {
        Box::pin(async move {
            if let RulePlan::Recursive {
                name,
                seed,
                step,
                is_distinct,
                depth_bound,
            } = source
            {
                self.recursive(
                    source,
                    name,
                    seed,
                    step,
                    *is_distinct,
                    *depth_bound,
                    binders,
                    traces,
                )
                .await
            } else {
                for child in source.children() {
                    self.expand(child, binders, traces).await?;
                }
                Ok(())
            }
        })
    }

    fn lower(
        &self,
        source: &RulePlan,
        binders: &[(String, Planned)],
        traces: &[(String, Vec<Trace>)],
    ) -> Result<LoweredClosure, RuleError> {
        let mut compiler = Compiler {
            rule: self.rule,
            binding: self.ports,
            session: &self.session,
            registry: self.registry,
            binders: binders.to_vec(),
            trace_binders: traces.to_vec(),
            native: &self.native,
            checks: vec![],
            recursive_counter: 0,
            binding_counter: 0,
        };
        let output = compiler.lower(source)?;
        let support = compiler.trace(source)?;
        Ok((output, support, compiler.checks))
    }

    async fn materialize(&mut self, plan: LogicalPlan) -> Result<(LogicalPlan, usize), RuleError> {
        let completed = self
            .session
            .prepare_rule_plan(plan, self.cancel)?
            .execute(self.cancel)
            .await?;
        self.plans.push(completed.prepared().observation().clone());
        let count = completed
            .batches()
            .iter()
            .try_fold(0usize, |n, batch| n.checked_add(batch.num_rows()))
            .ok_or_else(|| internal("recursive row count overflow"))?;
        let role = format!("{}:{}", self.prefix, self.counter);
        self.counter = self
            .counter
            .checked_add(1)
            .ok_or_else(|| internal("recursive role count overflow"))?;
        self.session = self
            .session
            .with_computation_roles(BTreeMap::from([(role.clone(), completed)]), self.cancel)?;
        Ok((self.session.scan_computation_role(&role)?, count))
    }

    async fn empty(&mut self, plan: LogicalPlan) -> Result<bool, RuleError> {
        let plan = LogicalPlanBuilder::from(plan)
            .limit(0, Some(1))
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        let completed = self
            .session
            .prepare_rule_plan(plan, self.cancel)?
            .execute(self.cancel)
            .await?;
        self.plans.push(completed.prepared().observation().clone());
        Ok(completed
            .batches()
            .iter()
            .all(|batch| batch.num_rows() == 0))
    }

    async fn checks(&mut self, checks: Vec<(LogicalPlan, String)>) -> Result<(), RuleError> {
        for (plan, reason) in checks {
            if !self.empty(plan).await? {
                return Err(internal(reason));
            }
        }
        Ok(())
    }

    async fn support(
        &mut self,
        incoming: Vec<Trace>,
        current: &[Trace],
    ) -> Result<(Vec<Trace>, bool), RuleError> {
        let mut grouped: BTreeMap<TraceKey, (Trace, Vec<LogicalPlan>)> = BTreeMap::new();
        for trace in current.iter().cloned().chain(incoming) {
            let key = (
                trace.source.relation,
                trace.source.port,
                trace.source.absence,
            );
            grouped
                .entry(key)
                .or_insert_with(|| (trace.clone(), vec![]))
                .1
                .push(trace.output.plan);
        }
        let mut result = vec![];
        let mut changed = false;
        for ((relation, port, absence), (mut trace, plans)) in grouped {
            let (plan, count) = self.materialize(union(plans)?).await?;
            let previous = current.iter().find(|old| {
                old.source.relation == relation
                    && old.source.port == port
                    && old.source.absence == absence
            });
            changed |= if let Some(previous) = previous {
                !self
                    .empty(exact_difference(
                        plan.clone(),
                        previous.output.plan.clone(),
                    )?)
                    .await?
            } else {
                count != 0
            };
            trace.output.plan = plan;
            for column in &mut trace.output.columns {
                column.qualifier = None;
                column.physical = None;
            }
            result.push(trace);
        }
        Ok((result, changed))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "lexical recursion retains its actual seed, step, policy and outer native bindings"
    )]
    async fn recursive(
        &mut self,
        source: &RulePlan,
        name: &str,
        seed: &RulePlan,
        step: &RulePlan,
        distinct: bool,
        bound: DepthBound,
        outer: &[(String, Planned)],
        outer_traces: &[(String, Vec<Trace>)],
    ) -> Result<(), RuleError> {
        self.expand(seed, outer, outer_traces).await?;
        let (mut facts, seed_support, checks) = self.lower(seed, outer, outer_traces)?;
        self.checks(checks).await?;
        let initial = if distinct {
            union([facts.plan.clone()])?
        } else {
            facts.plan.clone()
        };
        let (plan, seed_rows) = self.materialize(initial).await?;
        facts.plan = plan;
        for column in &mut facts.columns {
            column.qualifier = None;
            column.physical = None;
        }
        let mut delta = facts.clone();
        let (mut supports, _) = self.support(seed_support, &[]).await?;
        let bound = match bound {
            DepthBound::FixedPoint => None,
            DepthBound::SeedRows => Some(seed_rows),
            DepthBound::Bounded(value) => Some(value as usize),
        };
        let mut settled = false;
        for round in 0..self.max_rounds {
            self.cancel
                .checkpoint()
                .map_err(pse_catalog::CatalogError::from)?;
            let mut binders: Binders = outer.to_vec();
            binders.push((
                name.to_owned(),
                if distinct {
                    facts.clone()
                } else {
                    delta.clone()
                },
            ));
            let mut trace_binders: TraceBinders = outer_traces.to_vec();
            trace_binders.push((name.to_owned(), supports.clone()));
            self.expand(step, &binders, &trace_binders).await?;
            let (candidate, step_support, checks) = self.lower(step, &binders, &trace_binders)?;
            compatible(&facts, &candidate)?;
            self.checks(checks).await?;
            let candidates = if distinct {
                let plans = super::delta::substitute_inputs(
                    &candidate.plan,
                    &BTreeMap::from([(scan_name(&facts.plan)?, delta.plan.clone())]),
                )?;
                if plans.is_empty() {
                    candidate.plan.clone()
                } else {
                    union(plans)?
                }
            } else {
                candidate.plan.clone()
            };
            let candidates = if distinct {
                exact_difference(candidates, facts.plan.clone())?
            } else {
                candidates
            };
            let (added, count) = self.materialize(candidates).await?;
            if count != 0 && bound.is_some_and(|bound| round as usize >= bound) {
                return Err(internal(format!(
                    "recursive rule {name} has pending rows beyond its declared depth bound"
                )));
            }
            let unioned = LogicalPlanBuilder::from(facts.plan.clone())
                .union(added.clone())
                .and_then(LogicalPlanBuilder::build)
                .map_err(engine)?;
            let (next, _) = self
                .materialize(if distinct { union([unioned])? } else { unioned })
                .await?;
            delta.plan = added;
            facts.plan = next;
            let (next_support, support_changed) = self.support(step_support, &supports).await?;
            if support_changed && bound.is_some_and(|bound| round as usize >= bound) {
                return Err(internal(format!(
                    "recursive rule {name} has pending source support beyond its declared depth bound"
                )));
            }
            supports = next_support;
            if count == 0 && !support_changed {
                settled = true;
                break;
            }
        }
        if !settled {
            return Err(RuleError::ResourceLimit {
                consumer: format!("nested recursive rule {name} has unfinished finite work"),
                config_keys: vec!["pse.rules.max_rounds".to_owned()],
            });
        }
        self.native.results.push((source.clone(), facts));
        self.native.traces.push((source.clone(), supports));
        Ok(())
    }
}

fn exact_difference(left: LogicalPlan, right: LogicalPlan) -> Result<LogicalPlan, RuleError> {
    let columns = left
        .schema()
        .fields()
        .iter()
        .map(|field| field.name().to_owned())
        .collect::<Vec<_>>();
    difference(
        left,
        right,
        &columns.iter().map(String::as_str).collect::<Vec<_>>(),
    )
}

fn scan_name(plan: &LogicalPlan) -> Result<datafusion_common::TableReference, RuleError> {
    if let LogicalPlan::TableScan(scan) = plan {
        Ok(scan.table_name.clone())
    } else {
        Err(internal(
            "recursive materialization is not its actual retained scan",
        ))
    }
}

type LoweredClosure = (Planned, Vec<Trace>, Vec<(LogicalPlan, String)>);
