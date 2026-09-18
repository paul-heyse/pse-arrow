// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Recover row witnesses from the native logical plan before optimization.
//! Each occurrence carries actual typed source keys. Aggregates join their final
//! values to participating input groups, so lineage multiplicity cannot change sums.
mod shared;
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion_common::{Column, JoinType, NullEquality};
use datafusion_expr::{Expr, LogicalPlan, LogicalPlanBuilder};
use pse_schema::{
    Registry,
    model::{RelationKey, RuleSpec},
};
use shared::{Shared, Traces, traced};

#[derive(Clone, Debug)]
pub(crate) struct SupportSource {
    pub(crate) relation: RelationKey,
    pub(crate) port: &'static str,
    pub(crate) absence: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct SupportPlan {
    pub(crate) source: SupportSource,
    pub(crate) plan: LogicalPlan,
}

pub(crate) fn compile_support(
    plan: &LogicalPlan,
    rule: &RuleSpec,
    registry: &Registry,
    session: &pse_catalog::session::SnapshotSession,
    cancel: &pse_ids::CancellationToken,
) -> Result<Vec<SupportPlan>, RuleError> {
    shared::trace(plan, rule, registry, session, cancel)
}
fn columns(plan: &LogicalPlan) -> Vec<Expr> {
    plan.schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect()
}
fn hidden(plan: &LogicalPlan) -> Vec<Expr> {
    plan.schema()
        .iter()
        .filter(|(_, field)| field.name().starts_with("__pse_support_"))
        .map(|(qualifier, field)| Expr::Column(Column::new(qualifier.cloned(), field.name())))
        .collect()
}
fn project(plan: LogicalPlan, exprs: Vec<Expr>) -> Result<LogicalPlan, RuleError> {
    LogicalPlanBuilder::from(plan)
        .project(exprs)
        .map_err(engine)?
        .build()
        .map_err(engine)
}
fn rebuild(plan: &LogicalPlan, inputs: Vec<LogicalPlan>) -> Result<LogicalPlan, RuleError> {
    plan.with_new_exprs(plan.expressions(), inputs)
        .map_err(engine)
}
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive native logical-operator visitor keeps support semantics and refusals together"
)]
fn trace_node(
    plan: &LogicalPlan,
    rule: &RuleSpec,
    registry: &Registry,
    port: Option<&'static str>,
    shared: &mut Shared<'_>,
    children: &Traces,
) -> Result<Vec<SupportPlan>, RuleError> {
    match plan {
        LogicalPlan::TableScan(scan) => {
            let relation = format!(
                "{}.{}",
                scan.table_name.schema().unwrap_or_default(),
                scan.table_name.table()
            );
            let spec = registry
                .relation(&relation)
                .ok_or_else(|| internal("native support scan is undeclared"))?;
            let input = rule
                .inputs
                .iter()
                .find(|input| {
                    input.relation == relation && port.is_none_or(|port| input.port == port)
                })
                .ok_or_else(|| internal("native support input alias is undeclared"))?;
            if port.is_none()
                && rule
                    .inputs
                    .iter()
                    .filter(|input| input.relation == relation)
                    .map(|input| input.port)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != 1
            {
                return Err(internal(
                    "ambiguous native support input: use its declared port alias",
                ));
            }
            let mut exprs = columns(plan);
            for (position, key) in spec.primary_key.iter().enumerate() {
                let (qualifier, field) = plan
                    .schema()
                    .qualified_field_with_unqualified_name(key)
                    .map_err(engine)?;
                exprs.push(
                    Expr::Column(Column::new(qualifier.cloned(), field.name()))
                        .alias(format!("__pse_support_{position}")),
                );
            }
            Ok(vec![SupportPlan {
                source: SupportSource {
                    relation: spec.key,
                    port: input.port,
                    absence: false,
                },
                plan: project(plan.clone(), exprs)?,
            }])
        }
        LogicalPlan::SubqueryAlias(alias) => {
            let port = rule
                .inputs
                .iter()
                .find(|input| input.port == alias.alias.table())
                .map(|input| input.port)
                .or(port);
            traced(&alias.input, port, children)?
                .into_iter()
                .map(|mut trace| {
                    trace.plan = rebuild(plan, vec![trace.plan])?;
                    Ok(trace)
                })
                .collect()
        }
        LogicalPlan::Projection(projection) => traced(&projection.input, port, children)?
            .into_iter()
            .map(|mut trace| {
                let mut exprs = projection.expr.clone();
                exprs.extend(hidden(&trace.plan));
                trace.plan = project(trace.plan, exprs)?;
                Ok(trace)
            })
            .collect(),
        LogicalPlan::Distinct(datafusion_expr::Distinct::All(input)) => {
            traced(input, port, children)
        }
        LogicalPlan::Aggregate(aggregate) => {
            let mut output = Vec::new();
            for mut trace in traced(&aggregate.input, port, children)? {
                trace.plan =
                    group_members(&shared.original(plan)?, &aggregate.group_expr, trace.plan)?;
                output.push(trace);
            }
            if aggregate.group_expr.is_empty() {
                let empty = LogicalPlanBuilder::from(shared.original(&aggregate.input)?)
                    .aggregate(
                        Vec::<Expr>::new(),
                        vec![
                            datafusion::functions_aggregate::expr_fn::count(datafusion_expr::lit(
                                1i64,
                            ))
                            .alias("__pse_count"),
                        ],
                    )
                    .map_err(engine)?
                    .filter(datafusion_expr::col("__pse_count").eq(datafusion_expr::lit(0i64)))
                    .map_err(engine)?
                    .build()
                    .map_err(engine)?;
                let absence = LogicalPlanBuilder::from(shared.original(plan)?)
                    .cross_join(empty)
                    .map_err(engine)?
                    .project(columns(plan))
                    .map_err(engine)?
                    .build()
                    .map_err(engine)?;
                for mut source in scopes(&aggregate.input, rule, registry, port, shared)? {
                    source.absence = true;
                    output.push(SupportPlan {
                        source,
                        plan: absence.clone(),
                    });
                }
            }
            Ok(output)
        }
        LogicalPlan::Window(window) => {
            // A window observes its complete partition, including ordering and frame
            // selection. Retain that conservative dependency; never recompute a
            // window over rows expanded by witness multiplicity.
            let mut output = Vec::new();
            for expression in &window.window_expr {
                let mut expression = expression;
                while let Expr::Alias(alias) = expression {
                    expression = &alias.expr;
                }
                let Expr::WindowFunction(function) = expression else {
                    return Err(internal("window expression lacks its native function"));
                };
                for mut trace in traced(&window.input, port, children)? {
                    trace.plan = match_members(
                        &shared.original(plan)?,
                        &function.params.partition_by,
                        &function.params.partition_by,
                        trace.plan,
                    )?;
                    output.push(trace);
                }
            }
            Ok(output)
        }
        LogicalPlan::Limit(limit) => {
            let keys = columns(&limit.input);
            traced(&limit.input, port, children)?
                .into_iter()
                .map(|mut trace| {
                    trace.plan = match_members(&shared.original(plan)?, &keys, &keys, trace.plan)?;
                    Ok(trace)
                })
                .collect()
        }
        LogicalPlan::Union(union) => {
            let mut output = Vec::new();
            for child in &union.inputs {
                for mut trace in traced(child, port, children)? {
                    let mut exprs = child
                        .schema()
                        .columns()
                        .into_iter()
                        .zip(plan.schema().iter())
                        .map(|(source, (qualifier, field))| {
                            Expr::Column(source).alias_qualified(qualifier.cloned(), field.name())
                        })
                        .collect::<Vec<_>>();
                    exprs.extend(hidden(&trace.plan));
                    trace.plan = project(trace.plan, exprs)?;
                    output.push(trace);
                }
            }
            Ok(output)
        }
        LogicalPlan::Join(join) => {
            let mut output = Vec::new();
            for (index, child) in [&join.left, &join.right].into_iter().enumerate() {
                let absent = (join.join_type == JoinType::LeftAnti && index == 1)
                    || (join.join_type == JoinType::RightAnti && index == 0);
                if absent {
                    for mut source in scopes(child, rule, registry, port, shared)? {
                        source.absence = true;
                        output.push(SupportPlan {
                            source,
                            plan: shared.original(plan)?,
                        });
                    }
                    continue;
                }
                let nullable = join.join_type == JoinType::Full
                    || (join.join_type == JoinType::Left && index == 1)
                    || (join.join_type == JoinType::Right && index == 0);
                if nullable {
                    // Presence belongs to the actual input, not to a possibly
                    // expanded witness branch. An unmatched outer row records
                    // absence of that exact scope, never a fabricated null key.
                    let mut marked = columns(child);
                    marked.push(datafusion_expr::lit(true).alias("__pse_outer_present"));
                    let marked = project(shared.original(child)?, marked)?;
                    let inputs = if index == 0 {
                        vec![marked, shared.original(&join.right)?]
                    } else {
                        vec![shared.original(&join.left)?, marked]
                    };
                    let absent = LogicalPlanBuilder::from(rebuild(plan, inputs)?)
                        .filter(datafusion_expr::col("__pse_outer_present").is_null())
                        .map_err(engine)?
                        .project(columns(plan))
                        .map_err(engine)?
                        .build()
                        .map_err(engine)?;
                    for mut source in scopes(child, rule, registry, port, shared)? {
                        source.absence = true;
                        output.push(SupportPlan {
                            source,
                            plan: absent.clone(),
                        });
                    }
                }
                for mut trace in traced(child, port, children)? {
                    if nullable {
                        let mut marked = columns(&trace.plan);
                        marked.push(datafusion_expr::lit(true).alias("__pse_outer_present"));
                        trace.plan = project(trace.plan, marked)?;
                    }
                    let inputs = if index == 0 {
                        vec![trace.plan, shared.original(&join.right)?]
                    } else {
                        vec![shared.original(&join.left)?, trace.plan]
                    };
                    let mut native = plan.clone();
                    if let LogicalPlan::Join(join) = &mut native
                        && matches!(join.join_type, JoinType::LeftSemi | JoinType::RightSemi)
                    {
                        join.join_type = JoinType::Inner;
                    }
                    let traced = rebuild(&native, inputs)?;
                    let traced = if nullable {
                        LogicalPlanBuilder::from(traced)
                            .filter(datafusion_expr::col("__pse_outer_present").is_not_null())
                            .map_err(engine)?
                            .build()
                            .map_err(engine)?
                    } else {
                        traced
                    };
                    let mut exprs = columns(plan);
                    exprs.extend(hidden(&traced));
                    trace.plan = project(traced, exprs)?;
                    output.push(trace);
                }
            }
            Ok(output)
        }
        LogicalPlan::Filter(_)
        | LogicalPlan::Sort(_)
        | LogicalPlan::Repartition(_)
        | LogicalPlan::Unnest(_)
        | LogicalPlan::Subquery(_) => {
            let inputs = plan.inputs();
            let input = inputs
                .first()
                .ok_or_else(|| internal("native unary support input absent"))?;
            traced(input, port, children)?
                .into_iter()
                .map(|mut trace| {
                    trace.plan = rebuild(plan, vec![trace.plan])?;
                    Ok(trace)
                })
                .collect()
        }
        // Value-only roots have no source witnesses. They still produce assertions.
        LogicalPlan::EmptyRelation(_) | LogicalPlan::Values(_) => Ok(Vec::new()),
        // This is a domain lineage obligation, not a restriction on session SQL.
        // Unsupported recovery cannot silently fabricate or discard source evidence.
        other => Err(internal(format!(
            "native source support needs a recovery rule for {}",
            other.display()
        ))),
    }
}
fn scopes(
    plan: &LogicalPlan,
    rule: &RuleSpec,
    registry: &Registry,
    port: Option<&'static str>,
    shared: &mut Shared<'_>,
) -> Result<Vec<SupportSource>, RuleError> {
    let mut out = Vec::new();
    let mut pending = vec![(plan, port)];
    let mut visited = std::collections::HashSet::new();
    while let Some((node, port)) = pending.pop() {
        if !visited.insert((std::ptr::from_ref(node) as usize, port)) {
            continue;
        }
        let children = node.inputs();
        shared.charge((children.len() + 1).saturating_mul(128))?;
        if let LogicalPlan::TableScan(scan) = node {
            let name = format!(
                "{}.{}",
                scan.table_name.schema().unwrap_or_default(),
                scan.table_name.table()
            );
            let spec = registry
                .relation(&name)
                .ok_or_else(|| internal("absence scan is undeclared"))?;
            for input in &rule.inputs {
                if input.relation == name
                    && port.is_none_or(|port| input.port == port)
                    && !out.iter().any(|value: &SupportSource| {
                        value.relation == spec.key && value.port == input.port
                    })
                {
                    out.push(SupportSource {
                        relation: spec.key,
                        port: input.port,
                        absence: true,
                    });
                }
            }
        }
        let port = shared::child_port(node, rule, port);
        pending.extend(children.into_iter().rev().map(|child| (child, port)));
    }
    Ok(out)
}
fn group_members(
    aggregate: &LogicalPlan,
    groups: &[Expr],
    members: LogicalPlan,
) -> Result<LogicalPlan, RuleError> {
    let head_keys = columns(aggregate)
        .into_iter()
        .take(groups.len())
        .collect::<Vec<_>>();
    match_members(aggregate, &head_keys, groups, members)
}
fn match_members(
    normal: &LogicalPlan,
    head_keys: &[Expr],
    member_keys: &[Expr],
    members: LogicalPlan,
) -> Result<LogicalPlan, RuleError> {
    let mut member_exprs = member_keys
        .iter()
        .enumerate()
        .map(|(index, expr)| expr.clone().alias(format!("__pse_group_{index}")))
        .collect::<Vec<_>>();
    member_exprs.extend(hidden(&members));
    let members = LogicalPlanBuilder::from(project(members, member_exprs)?)
        .alias("__pse_members")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let mut head_exprs = columns(normal)
        .into_iter()
        .enumerate()
        .map(|(index, expr)| expr.alias(format!("__pse_value_{index}")))
        .collect::<Vec<_>>();
    head_exprs.extend(
        head_keys
            .iter()
            .enumerate()
            .map(|(index, expr)| expr.clone().alias(format!("__pse_group_{index}"))),
    );
    let head = LogicalPlanBuilder::from(project(normal.clone(), head_exprs)?)
        .alias("__pse_head")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let builder = LogicalPlanBuilder::from(head);
    let joined = if head_keys.is_empty() {
        builder.cross_join(members).map_err(engine)?
    } else {
        builder
            .join_detailed(
                members,
                JoinType::Inner,
                (
                    (0..head_keys.len())
                        .map(|index| {
                            Column::new(Some("__pse_head"), format!("__pse_group_{index}"))
                        })
                        .collect::<Vec<_>>(),
                    (0..member_keys.len())
                        .map(|index| {
                            Column::new(Some("__pse_members"), format!("__pse_group_{index}"))
                        })
                        .collect::<Vec<_>>(),
                ),
                None,
                NullEquality::NullEqualsNull,
            )
            .map_err(engine)?
    }
    .build()
    .map_err(engine)?;
    let mut exprs = normal
        .schema()
        .iter()
        .enumerate()
        .map(|(index, (qualifier, field))| {
            Expr::Column(Column::new(
                Some("__pse_head"),
                format!("__pse_value_{index}"),
            ))
            .alias_qualified(qualifier.cloned(), field.name())
        })
        .collect::<Vec<_>>();
    exprs.extend(hidden(&joined));
    project(joined, exprs)
}

#[cfg(test)]
mod tests;
