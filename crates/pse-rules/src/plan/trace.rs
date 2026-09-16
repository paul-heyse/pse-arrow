// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Relational lineage plans retain actual matched source keys through projections.
//! One sidecar per scan avoids serializing typed keys inside the query engine.
use super::{Compiler, Planned, PortBinding, expr, head};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion_common::{Column as EngineColumn, JoinType, NullEquality};
use datafusion_expr::{Expr, LogicalPlan, LogicalPlanBuilder, col};
use pse_catalog::session::SnapshotSession;
use pse_schema::{
    Registry,
    model::{RelationKey, RulePlan, RuleSpec},
};
use std::collections::BTreeSet;

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
#[derive(Clone)]
pub(super) struct Trace {
    pub(super) source: SupportSource,
    pub(super) output: Planned,
}

/// Absence belongs to an exact input scope, irrespective of how often its scan
/// occurs in this lookup. All matching row witnesses remain occurrence-sensitive.
fn absence_scopes(plan: &RulePlan) -> BTreeSet<(&str, &'static str)> {
    plan.dependencies()
        .into_iter()
        .map(|(relation, port, _)| (relation, port))
        .collect()
}

pub(crate) fn compile_support_bound(
    rule: &RuleSpec,
    binding: &PortBinding,
    session: &SnapshotSession,
    registry: &Registry,
    native: &super::NativeBindings,
) -> Result<Vec<SupportPlan>, RuleError> {
    let mut compiler = Compiler {
        rule,
        binding,
        session,
        registry,
        binders: vec![],
        trace_binders: vec![],
        native,
        checks: vec![],
        recursive_counter: 0,
        binding_counter: 0,
    };
    let target = registry
        .relation(rule.head.relation())
        .ok_or_else(|| internal("missing trace head"))?;
    let traces = compiler.trace(&rule.plan)?;
    traces
        .into_iter()
        .map(|trace| {
            let (plan, _, _) = head::prepare(
                rule,
                trace.output,
                target.columns.clone(),
                registry,
                true,
                session,
            )?;
            Ok(SupportPlan {
                source: trace.source,
                plan,
            })
        })
        .collect()
}

impl Compiler<'_> {
    pub(super) fn trace(&mut self, source: &RulePlan) -> Result<Vec<Trace>, RuleError> {
        if let Some((_, output)) = self
            .native
            .traces
            .iter()
            .rev()
            .find(|(plan, _)| super::identity::same_plan(plan, source))
        {
            return Ok(output.clone());
        }
        match source {
            RulePlan::Assert { .. } => Err(internal(
                "assertion predicate must be split before source support lowering",
            )),
            RulePlan::Scan { relation, port } => self.trace_scan(source, relation, port),
            RulePlan::Union(inputs) => {
                let mut traces = vec![];
                for input in inputs {
                    traces.extend(self.trace(input)?);
                }
                Ok(traces)
            }
            RulePlan::EquiJoin {
                left,
                right,
                keys,
                null_equality,
            } => self.trace_join(left, right, keys, *null_equality),
            RulePlan::AntiJoin { left, right, keys } => self.trace_anti(source, left, right, keys),
            RulePlan::Aggregate { input, group, .. } => self.trace_aggregate(source, input, group),
            RulePlan::Filter { input, predicate } => {
                self.trace_unary(input, |input| RulePlan::Filter {
                    input,
                    predicate: predicate.clone(),
                })
            }
            RulePlan::Project { input, columns } => {
                self.trace_unary(input, |input| RulePlan::Project {
                    input,
                    columns: columns.clone(),
                })
            }
            RulePlan::Distinct(input) => self.trace_unary(input, RulePlan::Distinct),
            RulePlan::Unnest {
                input,
                column,
                value_name,
                null_list,
                empty_list,
            } => self.trace_unary(input, |input| RulePlan::Unnest {
                input,
                column: column.clone(),
                value_name: value_name.clone(),
                null_list: *null_list,
                empty_list: *empty_list,
            }),
            RulePlan::RecursiveRef { name }
                if self.trace_binders.iter().any(|(bound, _)| bound == name) =>
            {
                let traces = self
                    .trace_binders
                    .iter()
                    .rev()
                    .find(|(bound, _)| bound == name)
                    .ok_or_else(|| internal("recursive support binding absent"))?
                    .1
                    .clone();
                let normal = self.lower(source)?;
                let columns = normal
                    .columns
                    .iter()
                    .map(|column| column.name.clone())
                    .collect::<Vec<_>>();
                traces
                    .into_iter()
                    .map(|mut trace| {
                        trace.output = aggregate_members(&normal, trace.output, &columns)?;
                        Ok(trace)
                    })
                    .collect()
            }
            RulePlan::Recursive { .. } | RulePlan::RecursiveRef { .. } => Err(internal(
                "native recursive computation is missing its bound result and source-support relations",
            )),
        }
    }
    fn trace_scan(
        &mut self,
        source: &RulePlan,
        relation: &str,
        port: &'static str,
    ) -> Result<Vec<Trace>, RuleError> {
        let mut output = self.lower(source)?;
        let spec = self
            .registry
            .relation(relation)
            .ok_or_else(|| internal("undeclared support scan"))?;
        let mut expressions = output
            .columns
            .iter()
            .map(expr::column_expression)
            .collect::<Vec<_>>();
        for (ordinal, key) in spec.primary_key.iter().enumerate() {
            let name = format!("__pse_support_{ordinal}");
            if output.columns.iter().any(|column| column.name == name) {
                return Err(internal("source column collides with support bookkeeping"));
            }
            expressions.push(expr::column_expression(expr::lookup(&output, key)?).alias(&name));
            output.hidden.push(name);
        }
        output.plan = LogicalPlanBuilder::from(output.plan)
            .project(expressions)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        Ok(vec![Trace {
            source: SupportSource {
                relation: spec.key,
                port,
                absence: false,
            },
            output,
        }])
    }
    fn trace_join(
        &mut self,
        left: &RulePlan,
        right: &RulePlan,
        keys: &[(
            std::borrow::Cow<'static, str>,
            std::borrow::Cow<'static, str>,
        )],
        null_equality: pse_schema::model::NullEquality,
    ) -> Result<Vec<Trace>, RuleError> {
        let normal_left = self.lower(left)?;
        let normal_right = self.lower(right)?;
        let mut traces = vec![];
        for mut trace in self.trace(left)? {
            let plan = RulePlan::EquiJoin {
                left: Box::new(bound("__trace_left")),
                right: Box::new(bound("__trace_right")),
                keys: keys.to_vec(),
                null_equality,
            };
            trace.output = self.bound_lower(
                &plan,
                vec![
                    ("__trace_left", trace.output),
                    ("__trace_right", normal_right.clone()),
                ],
            )?;
            traces.push(trace);
        }
        for mut trace in self.trace(right)? {
            let plan = RulePlan::EquiJoin {
                left: Box::new(bound("__trace_left")),
                right: Box::new(bound("__trace_right")),
                keys: keys.to_vec(),
                null_equality,
            };
            trace.output = self.bound_lower(
                &plan,
                vec![
                    ("__trace_left", normal_left.clone()),
                    ("__trace_right", trace.output),
                ],
            )?;
            traces.push(trace);
        }
        Ok(traces)
    }
    fn trace_anti(
        &mut self,
        source: &RulePlan,
        left: &RulePlan,
        right: &RulePlan,
        keys: &[(
            std::borrow::Cow<'static, str>,
            std::borrow::Cow<'static, str>,
        )],
    ) -> Result<Vec<Trace>, RuleError> {
        let normal_right = self.lower(right)?;
        let mut traces = vec![];
        for mut trace in self.trace(left)? {
            let plan = RulePlan::AntiJoin {
                left: Box::new(bound("__trace_left")),
                right: Box::new(bound("__trace_right")),
                keys: keys.to_vec(),
            };
            trace.output = self.bound_lower(
                &plan,
                vec![
                    ("__trace_left", trace.output),
                    ("__trace_right", normal_right.clone()),
                ],
            )?;
            traces.push(trace);
        }
        for (relation, port) in absence_scopes(right) {
            let spec = self
                .registry
                .relation(relation)
                .ok_or_else(|| internal("undeclared absence relation"))?;
            traces.push(Trace {
                source: SupportSource {
                    relation: spec.key,
                    port,
                    absence: true,
                },
                output: self.lower(source)?,
            });
        }
        Ok(traces)
    }
    fn trace_aggregate(
        &mut self,
        source: &RulePlan,
        input: &RulePlan,
        group: &[std::borrow::Cow<'static, str>],
    ) -> Result<Vec<Trace>, RuleError> {
        let normal = self.lower(source)?;
        let mut traces = vec![];
        for trace in self.trace(input)? {
            traces.push(Trace {
                source: trace.source,
                output: aggregate_members(&normal, trace.output, group)?,
            });
        }
        if group.is_empty() {
            let input_plan = self.lower(input)?.plan;
            let empty = LogicalPlanBuilder::from(input_plan)
                .aggregate(
                    Vec::<Expr>::new(),
                    vec![
                        datafusion::functions_aggregate::expr_fn::count(datafusion_expr::lit(1u64))
                            .alias("__pse_empty_count"),
                    ],
                )
                .map_err(engine)?
                .filter(col("__pse_empty_count").eq(datafusion_expr::lit(0i64)))
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            let plan = LogicalPlanBuilder::from(normal.plan.clone())
                .cross_join(empty)
                .map_err(engine)?
                .project(normal.columns.iter().map(expr::column_expression))
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            for (relation, port) in absence_scopes(input) {
                let spec = self
                    .registry
                    .relation(relation)
                    .ok_or_else(|| internal("undeclared aggregate absence relation"))?;
                traces.push(Trace {
                    source: SupportSource {
                        relation: spec.key,
                        port,
                        absence: true,
                    },
                    output: Planned {
                        plan: plan.clone(),
                        columns: normal.columns.clone(),
                        hidden: vec![],
                    },
                });
            }
        }
        Ok(traces)
    }
    fn trace_unary(
        &mut self,
        input: &RulePlan,
        node: impl Fn(Box<RulePlan>) -> RulePlan,
    ) -> Result<Vec<Trace>, RuleError> {
        let mut result = vec![];
        for mut trace in self.trace(input)? {
            trace.output = self.bound_lower(
                &node(Box::new(bound("__trace_input"))),
                vec![("__trace_input", trace.output)],
            )?;
            result.push(trace);
        }
        Ok(result)
    }
    fn bound_lower(
        &mut self,
        plan: &RulePlan,
        bindings: Vec<(&str, Planned)>,
    ) -> Result<Planned, RuleError> {
        let start = self.binders.len();
        self.binders.extend(
            bindings
                .into_iter()
                .map(|(name, output)| (name.to_owned(), output)),
        );
        let result = self.lower(plan);
        self.binders.truncate(start);
        result
    }
}
fn bound(name: &'static str) -> RulePlan {
    RulePlan::RecursiveRef { name }
}

fn aggregate_members(
    head: &Planned,
    members: Planned,
    group: &[std::borrow::Cow<'static, str>],
) -> Result<Planned, RuleError> {
    let group_names = group
        .iter()
        .map(|name| expr::lookup(&members, name).map(|column| column.name.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    let head_plan = LogicalPlanBuilder::from(head.plan.clone())
        .alias("__aggregate_head")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let member_plan = LogicalPlanBuilder::from(members.plan)
        .alias("__aggregate_member")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let builder = LogicalPlanBuilder::from(head_plan);
    let builder = if group.is_empty() {
        builder.cross_join(member_plan).map_err(engine)?
    } else {
        builder
            .join_detailed(
                member_plan,
                JoinType::Inner,
                (
                    group_names
                        .iter()
                        .map(|name| EngineColumn::new(Some("__aggregate_head"), name.as_ref()))
                        .collect::<Vec<_>>(),
                    group_names
                        .iter()
                        .map(|name| EngineColumn::new(Some("__aggregate_member"), name.as_ref()))
                        .collect::<Vec<_>>(),
                ),
                None,
                NullEquality::NullEqualsNull,
            )
            .map_err(engine)?
    };
    let mut projection = head
        .columns
        .iter()
        .map(|column| {
            Expr::Column(EngineColumn::new(
                Some("__aggregate_head"),
                column.name.as_ref(),
            ))
            .alias(column.name.as_ref())
        })
        .collect::<Vec<_>>();
    projection.extend(
        members
            .hidden
            .iter()
            .map(|name| col(format!("__aggregate_member.{name}")).alias(name)),
    );
    let plan = builder
        .project(projection)
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    Ok(Planned {
        plan,
        columns: head.columns.clone(),
        hidden: members.hidden,
    })
}
