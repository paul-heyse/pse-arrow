// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The bounded algebra maps to actual engine operators, not row-wise emulation.
use super::{Column, Compiler, Planned, expr};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion_common::{
    Column as EngineColumn, JoinType, NullEquality as EngineNullEquality, UnnestOptions,
};
use datafusion_expr::{ExprSchemable, LogicalPlanBuilder, col, lit};
use pse_schema::model::{
    AggregateEmptyPolicy, AggregateNullPolicy, ColumnSpec, LogicalType, NullEquality,
    NullListPolicy, RuleAggregateFn, RulePlan,
};

impl Compiler<'_> {
    pub(super) fn lower(&mut self, source: &RulePlan) -> Result<Planned, RuleError> {
        match source {
            RulePlan::Scan { relation, port } => self.scan(relation, port),
            RulePlan::RecursiveRef { name } => self
                .binders
                .iter()
                .rev()
                .find(|(candidate, _)| candidate == name)
                .map(|(_, planned)| planned.clone())
                .ok_or_else(|| internal("recursive reference is outside its lexical step")),
            RulePlan::Filter { input, predicate } => {
                let mut input = self.lower(input)?;
                let (predicate, ty) = expr::lower(predicate, &input, self.registry)?;
                expr::boolean(&ty)?;
                input.plan = LogicalPlanBuilder::from(input.plan)
                    .filter(predicate)
                    .map_err(engine)?
                    .build()
                    .map_err(engine)?;
                if let RulePlan::Filter { predicate, .. } = source {
                    refine_nonnull(predicate, &mut input.columns);
                }
                Ok(input)
            }
            RulePlan::Project { input, columns } => {
                let input = self.lower(input)?;
                let mut expressions = vec![];
                let mut contracts = vec![];
                for (name, expression) in columns {
                    let (value, mut contract) = expr::lower(expression, &input, self.registry)?;
                    contract.spec.name = name;
                    let field = pse_schema::arrow::field_for(self.registry, &contract.spec)
                        .map_err(|error| internal(error.to_string()))?;
                    expressions.push(value.alias_with_metadata(
                        *name,
                        Some(datafusion_common::metadata::FieldMetadata::from(&field)),
                    ));
                    contract.qualifier = None;
                    contracts.push(contract);
                }
                expressions.extend(input.hidden.iter().map(col));
                let plan = LogicalPlanBuilder::from(input.plan)
                    .project(expressions)
                    .map_err(engine)?
                    .build()
                    .map_err(engine)?;
                Ok(Planned {
                    plan,
                    columns: contracts,
                    hidden: input.hidden,
                })
            }
            RulePlan::EquiJoin {
                left,
                right,
                keys,
                null_equality,
            } => self.join(left, right, keys, false, *null_equality),
            RulePlan::AntiJoin { left, right, keys } => {
                self.join(left, right, keys, true, NullEquality::NullEqualsNothing)
            }
            RulePlan::Distinct(input) => {
                let mut input = self.lower(input)?;
                for column in &input.columns {
                    self.key(column)?;
                }
                input.plan = LogicalPlanBuilder::from(input.plan)
                    .distinct()
                    .map_err(engine)?
                    .build()
                    .map_err(engine)?;
                Ok(input)
            }
            RulePlan::Union(inputs) => self.union(inputs),
            RulePlan::Aggregate {
                input,
                group,
                aggregates,
            } => self.aggregate(input, group, aggregates),
            RulePlan::Unnest {
                input,
                column,
                value_name,
                null_list,
                ..
            } => self.unnest(input, column, value_name, *null_list),
            RulePlan::Recursive {
                name,
                seed,
                step,
                is_distinct,
                depth_bound,
            } => self.recursive(name, seed, step, *is_distinct, *depth_bound),
        }
    }
    fn union(&mut self, inputs: &[RulePlan]) -> Result<Planned, RuleError> {
        let mut inputs = inputs.iter();
        let mut output = self.lower(inputs.next().ok_or_else(|| internal("empty union"))?)?;
        for input in inputs {
            let input = self.lower(input)?;
            compatible(&output, &input)?;
            output.plan = LogicalPlanBuilder::from(output.plan)
                .union(input.plan)
                .map_err(engine)?
                .build()
                .map_err(engine)?;
        }
        Ok(output)
    }
    fn scan(&self, relation: &str, port: &str) -> Result<Planned, RuleError> {
        let key = self
            .binding
            .ports
            .get(port)
            .ok_or_else(|| internal(format!("unbound rule port {port}")))?;
        if key.qualified_name() != relation {
            return Err(internal(format!(
                "port {port} is bound to {key}, not {relation}"
            )));
        }
        let spec = self
            .registry
            .relation(relation)
            .ok_or_else(|| internal("scan relation is not declared"))?;
        if &spec.key != key {
            return Err(internal(
                "scan version differs from the exact registry declaration",
            ));
        }
        let source = self.session.table_source(key)?;
        let reference = self.session.table_reference(key)?;
        let plan = LogicalPlanBuilder::scan(reference, source, None)
            .map_err(engine)?
            .alias(port)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let columns = spec
            .columns
            .iter()
            .map(|spec| Column {
                spec: spec.clone(),
                qualifier: Some(port.to_owned()),
                literal: None,
            })
            .collect();
        Ok(Planned {
            plan,
            columns,
            hidden: vec![],
        })
    }
    pub(super) fn key(&self, column: &Column) -> Result<(), RuleError> {
        if column.spec.logical_type == LogicalType::F64 {
            return Err(RuleError::FloatKey {
                rule: self.rule.qualified_name(),
                column: column.spec.name.to_owned(),
            });
        }
        if !column.spec.logical_type.admits_exact_key() {
            return Err(internal("rule key has no admitted exact equality"));
        }
        Ok(())
    }
    fn join(
        &mut self,
        left: &RulePlan,
        right: &RulePlan,
        keys: &[(&str, &str)],
        anti: bool,
        null: NullEquality,
    ) -> Result<Planned, RuleError> {
        let mut left = self.lower(left)?;
        let right = self.lower(right)?;
        if keys.is_empty() {
            return Err(internal("rule joins require explicit equality keys"));
        }
        for (l, r) in keys {
            let left_key = expr::lookup(&left, l)?;
            let right_key = expr::lookup(&right, r)?;
            self.key(left_key)?;
            self.key(right_key)?;
            if left_key.spec.logical_type != right_key.spec.logical_type
                || left_key.spec.quantity != right_key.spec.quantity
            {
                return Err(internal("join key physical contracts differ"));
            }
        }
        let join_keys = (
            keys.iter()
                .map(|(l, _)| EngineColumn::from_qualified_name(*l))
                .collect::<Vec<_>>(),
            keys.iter()
                .map(|(_, r)| EngineColumn::from_qualified_name(*r))
                .collect::<Vec<_>>(),
        );
        left.plan = LogicalPlanBuilder::from(left.plan)
            .join_detailed(
                right.plan,
                if anti {
                    JoinType::LeftAnti
                } else {
                    JoinType::Inner
                },
                join_keys,
                None,
                match null {
                    NullEquality::NullEqualsNothing => EngineNullEquality::NullEqualsNothing,
                    NullEquality::NullEqualsNull => EngineNullEquality::NullEqualsNull,
                },
            )
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        if !anti {
            left.columns.extend(right.columns);
            left.hidden.extend(right.hidden);
        }
        Ok(left)
    }
    fn aggregate(
        &mut self,
        source: &RulePlan,
        group: &[&'static str],
        aggregates: &[pse_schema::model::RuleAggregate],
    ) -> Result<Planned, RuleError> {
        let input = self.lower(source)?;
        let mut groups = vec![];
        let mut output = vec![];
        for name in group {
            let mut column = expr::lookup(&input, name)?.clone();
            self.key(&column)?;
            groups.push(expr::column_expression(&column));
            column.qualifier = None;
            output.push(column);
        }
        groups.extend(input.hidden.iter().map(col));
        let mut expressions = vec![];
        for aggregate in aggregates {
            if aggregate.function != RuleAggregateFn::Count
                || aggregate.empty_policy != AggregateEmptyPolicy::Zero
                || !aggregate.order_by.is_empty()
            {
                return Err(internal(
                    "phase-0 supports unordered count with explicit zero for empty input",
                ));
            }
            let value = if let Some(source) = &aggregate.input {
                let (value, column) = expr::lower(source, &input, self.registry)?;
                if aggregate.null_policy == AggregateNullPolicy::Reject && column.spec.nullable {
                    let check = LogicalPlanBuilder::from(input.plan.clone())
                        .filter(value.clone().is_null())
                        .map_err(engine)?
                        .build()
                        .map_err(engine)?;
                    self.checks
                        .push((check, "aggregate rejects null input".to_owned()));
                }
                value
            } else {
                lit(1u64)
            };
            expressions.push(
                datafusion::functions_aggregate::expr_fn::count(value)
                    .cast_to(
                        &datafusion::arrow::datatypes::DataType::UInt64,
                        input.plan.schema(),
                    )
                    .map_err(engine)?
                    .alias(aggregate.output_name),
            );
            output.push(Column {
                spec: ColumnSpec::payload(
                    aggregate.output_name,
                    LogicalType::U64,
                    "Exact nonnegative row count",
                ),
                qualifier: None,
                literal: None,
            });
        }
        let plan = LogicalPlanBuilder::from(input.plan)
            .aggregate(groups, expressions)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        Ok(Planned {
            plan,
            columns: output,
            hidden: input.hidden,
        })
    }
    fn unnest(
        &mut self,
        source: &RulePlan,
        name: &str,
        value_name: &'static str,
        null: NullListPolicy,
    ) -> Result<Planned, RuleError> {
        let mut input = self.lower(source)?;
        let column = expr::lookup(&input, name)?;
        let (LogicalType::List(element) | LogicalType::FixedList(element, _)) =
            &column.spec.logical_type
        else {
            return Err(internal("unnest requires a declared list"));
        };
        let element = element.as_ref().clone();
        let value = expr::column_expression(column);
        if null == NullListPolicy::Reject && column.spec.nullable {
            let check = LogicalPlanBuilder::from(input.plan.clone())
                .filter(value.clone().is_null())
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            self.checks
                .push((check, "unnest rejects null lists".to_owned()));
        }
        let mut projected: Vec<_> = input.columns.iter().map(expr::column_expression).collect();
        projected.extend(input.hidden.iter().map(col));
        projected.push(value.alias(value_name));
        input.plan = LogicalPlanBuilder::from(input.plan)
            .project(projected)
            .map_err(engine)?
            .unnest_columns_with_options(
                vec![EngineColumn::from_name(value_name)],
                UnnestOptions::new().with_preserve_nulls(false),
            )
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        input.columns.push(Column {
            spec: ColumnSpec::payload(value_name, element, "Declared list member"),
            qualifier: None,
            literal: None,
        });
        let mut projected = input
            .columns
            .iter()
            .map(|column| {
                let field = pse_schema::arrow::field_for(self.registry, &column.spec)
                    .map_err(|error| internal(error.to_string()))?;
                Ok(expr::column_expression(column).alias_with_metadata(
                    column.spec.name,
                    Some(datafusion_common::metadata::FieldMetadata::from(&field)),
                ))
            })
            .collect::<Result<Vec<_>, RuleError>>()?;
        projected.extend(input.hidden.iter().map(col));
        input.plan = LogicalPlanBuilder::from(input.plan)
            .project(projected)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        for column in &mut input.columns {
            column.qualifier = None;
        }
        Ok(input)
    }
}
fn refine_nonnull(predicate: &pse_schema::model::RuleExpr, columns: &mut [Column]) {
    use pse_schema::model::RuleExpr;
    match predicate {
        RuleExpr::IsTrue(inner) => refine_nonnull(inner, columns),
        RuleExpr::And(parts) => {
            for part in parts {
                refine_nonnull(part, columns);
            }
        }
        RuleExpr::IsNotNull(inner) => {
            if let RuleExpr::Col(name) = inner.as_ref() {
                for column in columns {
                    if column.spec.name == *name
                        || column.qualifier.as_ref().is_some_and(|qualifier| {
                            *name == format!("{qualifier}.{}", column.spec.name)
                        })
                    {
                        column.spec.nullable = false;
                    }
                }
            }
        }
        _ => {}
    }
}
pub(super) fn compatible(left: &Planned, right: &Planned) -> Result<(), RuleError> {
    if left.columns.len() != right.columns.len()
        || left.columns.iter().zip(&right.columns).any(|(l, r)| {
            l.spec.name != r.spec.name
                || l.spec.logical_type != r.spec.logical_type
                || l.spec.quantity != r.spec.quantity
                || l.spec.nullable != r.spec.nullable
        })
        || left.hidden != right.hidden
    {
        return Err(internal(
            "union or recursive branches have different full column contracts",
        ));
    }
    Ok(())
}
