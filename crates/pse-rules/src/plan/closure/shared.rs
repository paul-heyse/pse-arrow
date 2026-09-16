// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Materialize actual immutable fan-out computations and their located witnesses.
//! This keeps relational support construction from duplicating an entire query for
//! every source occurrence. Each role owns its completed native result; no saved
//! graph or label supplies admission, and evolving heads retain their delta scans.

use super::{Closure, Planned, RuleError, RulePlan, Trace, TraceKey, engine};
use crate::{
    errmap::internal,
    plan::{expr, identity::same_plan},
};
use datafusion_common::Column;
use datafusion_expr::LogicalPlanBuilder;
use std::collections::{BTreeMap, BTreeSet};

pub(super) async fn prepare(closure: &mut Closure<'_>, root: &RulePlan) -> Result<(), RuleError> {
    for source in shared_subplans(root) {
        closure
            .cancel
            .checkpoint()
            .map_err(pse_catalog::CatalogError::from)?;
        if closure
            .native
            .results
            .iter()
            .any(|(plan, _)| same_plan(plan, source))
        {
            continue;
        }
        let started = std::time::Instant::now();
        let (output, traces, checks) = closure.lower(source, &[], &[])?;
        let mut output = addressable(output)?;
        let prepared = closure
            .session
            .prepare_rule_plan(output.plan.clone(), closure.cancel)?;
        // Sharing must not merge independent calls to stable or volatile functions.
        // The actual bound native functions decide this, not declaration spelling.
        if prepared.contains_volatile_expression() {
            continue;
        }
        closure.checks(checks).await?;
        let completed = prepared.execute(closure.cancel).await?;
        let has_rows = completed
            .batches()
            .iter()
            .any(|batch| batch.num_rows() != 0);
        closure.plans.push(completed.observation().clone());
        let role = format!("{}:{}", closure.prefix, closure.counter);
        closure.counter = closure
            .counter
            .checked_add(1)
            .ok_or_else(|| internal("shared computation role count overflow"))?;
        closure.session = closure
            .session
            .with_computation_roles(BTreeMap::from([(role.clone(), completed)]), closure.cancel)?;
        output.plan = closure.session.scan_computation_role(&role)?;
        let mut grouped: BTreeMap<TraceKey, (Trace, Vec<datafusion_expr::LogicalPlan>)> =
            BTreeMap::new();
        // A completed immutable empty result has no row witnesses. Consumers that
        // observe absence still derive their negative scope from the original
        // declaration and exact retained input ports.
        for mut trace in traces.into_iter().filter(|_| has_rows) {
            trace.output = addressable(trace.output)?;
            let key = (
                trace.source.relation,
                trace.source.port,
                trace.source.absence,
            );
            let plan = trace.output.plan.clone();
            grouped
                .entry(key)
                .or_insert_with(|| (trace, vec![]))
                .1
                .push(plan);
        }
        let mut supports = Vec::with_capacity(grouped.len());
        for (_, (mut trace, plans)) in grouped {
            // Keep all actual witness rows, including exact floating payload bits.
            // The final edge relation performs its declared identity deduplication.
            trace.output.plan = crate::strata::relational::union_all(plans)?
                .build()
                .map_err(engine)?;
            (trace.output.plan, _) = closure.materialize(trace.output.plan).await?;
            supports.push(trace);
        }
        tracing::debug!(
            rule = closure.rule.name,
            operation = source.op(),
            witnesses = supports.len(),
            elapsed_seconds = started.elapsed().as_secs_f64(),
            "shared native rule computation completed"
        );
        closure.native.results.push((source.clone(), output));
        closure.native.traces.push((source.clone(), supports));
    }
    Ok(())
}

/// Logical names remain available to the declaration frontend; physical columns
/// get unique names before Arrow materialization discards native qualifiers.
fn addressable(mut output: Planned) -> Result<Planned, RuleError> {
    let mut projection = Vec::with_capacity(output.columns.len() + output.hidden.len());
    for (ordinal, column) in output.columns.iter_mut().enumerate() {
        let name = format!("__pse_shared_column_{ordinal}");
        projection.push(expr::column_expression(column).alias(&name));
        column.physical = Some(Column::new_unqualified(name));
    }
    projection.extend(output.hidden.iter().map(datafusion_expr::col));
    output.plan = LogicalPlanBuilder::from(output.plan)
        .project(projection)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    Ok(output)
}

/// Count edges in the actual declaration DAG, not occurrences in its expanded
/// tree. A chain below a shared node needs only its fan-out boundary materialized.
fn shared_subplans(root: &RulePlan) -> Vec<&RulePlan> {
    fn inventory<'a>(plan: &'a RulePlan, nodes: &mut Vec<(&'a RulePlan, usize)>) {
        if let Some((_, uses)) = nodes.iter_mut().find(|(node, _)| same_plan(node, plan)) {
            *uses += 1;
            return;
        }
        nodes.push((plan, 1));
        for child in plan.children() {
            inventory(child, nodes);
        }
    }
    fn ordered<'a>(
        plan: &'a RulePlan,
        nodes: &[(&'a RulePlan, usize)],
        visited: &mut BTreeSet<usize>,
        selected: &mut Vec<&'a RulePlan>,
    ) {
        let Some(position) = nodes.iter().position(|(node, _)| same_plan(node, plan)) else {
            return;
        };
        if !visited.insert(position) {
            return;
        }
        for child in plan.children() {
            ordered(child, nodes, visited, selected);
        }
        // Recursion has its own lexical materialization and support closure.
        // Shared scopes here must be independent of any recursive binder.
        if nodes[position].1 > 1
            && !matches!(plan, RulePlan::Scan { .. } | RulePlan::RecursiveRef { .. })
            && !contains_recursion(plan)
        {
            selected.push(plan);
        }
    }
    let mut nodes = vec![];
    inventory(root, &mut nodes);
    let mut selected = vec![];
    ordered(root, &nodes, &mut BTreeSet::new(), &mut selected);
    selected
}

fn contains_recursion(plan: &RulePlan) -> bool {
    matches!(
        plan,
        RulePlan::Recursive { .. } | RulePlan::RecursiveRef { .. } | RulePlan::Assert { .. }
    ) || plan.children().into_iter().any(contains_recursion)
}
