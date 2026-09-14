// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite UNION ALL recursion retains a depth witness and refuses unfinished closure.
use super::{Compiler, Planned, expr, lower::compatible};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::catalog::cte_worktable::CteWorkTable;
use datafusion::datasource::provider_as_source;
use datafusion_expr::{LogicalPlan, LogicalPlanBuilder, col, lit};
use pse_schema::model::{DepthBound, RulePlan};
use std::sync::Arc;

impl Compiler<'_> {
    pub(super) fn recursive(
        &mut self,
        name: &str,
        seed: &RulePlan,
        step: &RulePlan,
        distinct: bool,
        bound: DepthBound,
    ) -> Result<Planned, RuleError> {
        if distinct || bound == DepthBound::FixedPoint {
            return Err(internal(
                "phase-0 recursion requires UNION ALL and a finite depth bound",
            ));
        }
        let seed = self.lower(seed)?;
        if !seed.hidden.is_empty() {
            return Err(internal(
                "nested recursive seeds may not capture an outer work table",
            ));
        }
        let ordinal = self.recursive_counter;
        self.recursive_counter += 1;
        let work_name = format!("pse_recursive_{ordinal}_{name}");
        let depth = format!("__pse_depth_{ordinal}");
        let limit = format!("__pse_limit_{ordinal}");
        let seeded = seed_with_bound(&seed, &depth, &limit, bound)?;
        let schema = Arc::new(seeded.schema().as_arrow().clone());
        let source = provider_as_source(Arc::new(CteWorkTable::new(&work_name, schema)));
        let work = LogicalPlanBuilder::scan(work_name.clone(), source, None)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let columns = seed
            .columns
            .iter()
            .map(|column| {
                let mut column = column.clone();
                column.qualifier = None;
                column
            })
            .collect();
        let bound_plan = Planned {
            plan: work,
            columns,
            hidden: vec![depth.clone(), limit.clone()],
        };
        self.binders.push((name.to_owned(), bound_plan.clone()));
        let step_result = self.lower(step);
        self.binders.pop();
        let step = step_result?;
        let mut expected = seed.clone();
        expected.hidden.clone_from(&bound_plan.hidden);
        compatible(&expected, &step)?;
        let mut projection: Vec<_> = step.columns.iter().map(expr::column_expression).collect();
        projection.push((col(&depth) + lit(1i64)).alias(&depth));
        projection.push(col(&limit));
        // Produce one extra depth to observe unfinished work, then stop physically.
        let step = LogicalPlanBuilder::from(step.plan)
            .filter(col(&depth).lt_eq(col(&limit)))
            .map_err(engine)?
            .project(projection)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let recursive = LogicalPlanBuilder::from(seeded)
            .to_recursive_query(work_name, step, false)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let overflow = LogicalPlanBuilder::from(recursive.clone())
            .filter(col(&depth).gt(col(&limit)))
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        self.checks.push((
            overflow,
            format!("recursive rule {name} has pending rows beyond its declared depth bound"),
        ));
        let projected = LogicalPlanBuilder::from(recursive)
            .project(seed.columns.iter().map(|column| col(column.spec.name)))
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        Ok(Planned {
            plan: projected,
            columns: seed
                .columns
                .into_iter()
                .map(|mut column| {
                    column.qualifier = None;
                    column
                })
                .collect(),
            hidden: vec![],
        })
    }
}

fn seed_with_bound(
    seed: &Planned,
    depth: &str,
    limit: &str,
    bound: DepthBound,
) -> Result<LogicalPlan, RuleError> {
    if seed
        .plan
        .schema()
        .fields()
        .iter()
        .any(|field| field.name() == depth || field.name() == limit)
    {
        return Err(internal(
            "rule column collides with compiler recursion bookkeeping",
        ));
    }
    let mut seed_expressions: Vec<_> = seed.columns.iter().map(expr::column_expression).collect();
    seed_expressions.push(lit(0i64).alias(depth));
    let seeded = LogicalPlanBuilder::from(seed.plan.clone())
        .project(seed_expressions)
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let seeded = match bound {
        DepthBound::SeedRows => {
            let count = LogicalPlanBuilder::from(seed.plan.clone())
                .aggregate(
                    Vec::<datafusion_expr::Expr>::new(),
                    vec![datafusion::functions_aggregate::expr_fn::count(lit(1i64)).alias(limit)],
                )
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            LogicalPlanBuilder::from(seeded)
                .cross_join(count)
                .map_err(engine)?
                .build()
                .map_err(engine)?
        }
        DepthBound::Bounded(value) if value > 0 => {
            let mut projection: Vec<_> = seeded
                .schema()
                .columns()
                .into_iter()
                .map(datafusion_expr::Expr::Column)
                .collect();
            projection.push(lit(i64::from(value)).alias(limit));
            LogicalPlanBuilder::from(seeded)
                .project(projection)
                .map_err(engine)?
                .build()
                .map_err(engine)?
        }
        _ => return Err(internal("recursive depth bound must be positive")),
    };
    Ok(seeded)
}
