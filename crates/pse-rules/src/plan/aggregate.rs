// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordered collection and checked integer reduction remain DataFusion aggregates.
use super::{Column, Compiler, Planned, expr};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::arrow::datatypes::DataType;
use datafusion::functions_aggregate::expr_fn::{count, sum};
use datafusion_common::ScalarValue;
use datafusion_expr::{
    Expr, ExprFunctionExt, ExprSchemable, LogicalPlan, LogicalPlanBuilder, col, expr::Sort, lit,
};
use pse_catalog::session::aggregate::{array_agg, max, min};
use pse_schema::model::{
    AggregateEmptyPolicy as Empty, AggregateNullPolicy as Null, FieldContract, FieldContract as T,
    RuleAggregate, RuleAggregateFn as F, RulePlan,
};

impl Compiler<'_> {
    pub(super) fn aggregate(
        &mut self,
        source: &RulePlan,
        group: &[std::borrow::Cow<'static, str>],
        aggregates: &[RuleAggregate],
    ) -> Result<Planned, RuleError> {
        let input = self.lower(source)?;
        let mut groups = vec![];
        let mut output = vec![];
        for name in group {
            let mut column = expr::lookup(&input, name)?.clone();
            self.key(&column)?;
            groups.push(expr::column_expression(&column));
            column.qualifier = None;
            column.physical = None;
            output.push(column);
        }
        groups.extend(input.hidden.iter().map(col));
        let mut expressions = vec![];
        let mut finishing = output
            .iter()
            .map(|column| col(column.name.as_ref()))
            .collect::<Vec<_>>();
        for aggregate in aggregates {
            let (value, column) = self.aggregate_input(&input, aggregate)?;
            let orders = self.aggregate_order(&input, &groups, aggregate)?;
            let (aggregate_expr, logical_type) =
                native_aggregate(&input, aggregate, &value, &column, orders)?;
            let mut spec =
                FieldContract::payload("_aggregate", logical_type, "Declared aggregate result");
            if aggregate.function != F::Count {
                spec = spec.with_quantity_contract(column.spec.quantity());
            }
            let accumulator_type = aggregate_expr
                .get_type(input.plan.schema())
                .map_err(engine)?;
            expressions.push(aggregate_expr.alias(aggregate.output_name.as_ref()));
            let finished = self.aggregate_empty(aggregate, &accumulator_type)?;
            finishing.push(finished.alias(aggregate.output_name.as_ref()));
            output.push(Column {
                name: aggregate.output_name.clone(),
                spec,
                qualifier: None,
                physical: None,
                literal: None,
            });
        }
        let grouped_plan = LogicalPlanBuilder::from(input.plan)
            .aggregate(groups, expressions)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        for aggregate in aggregates
            .iter()
            .filter(|aggregate| aggregate.empty_policy == Empty::Error)
        {
            let name = &aggregate.output_name;
            let count = aggregate.function == F::Count;
            let check = LogicalPlanBuilder::from(grouped_plan.clone())
                .filter(if count {
                    col(name.as_ref()).eq(lit(0i64))
                } else {
                    col(name.as_ref()).is_null()
                })
                .map_err(engine)?
                .project(vec![lit(true).alias("invalid_empty_group")])
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            self.checks
                .push((check, "aggregate rejects an empty group".to_owned()));
        }
        let plan = self.finish_aggregation(grouped_plan, finishing, &output, &input.hidden)?;
        Ok(Planned {
            plan,
            columns: output,
            hidden: input.hidden,
        })
    }

    fn aggregate_input(
        &mut self,
        input: &Planned,
        aggregate: &RuleAggregate,
    ) -> Result<(Expr, Column), RuleError> {
        let (value, column) = if let Some(source) = &aggregate.input {
            expr::lower(source, input, self.registry, self.session)?
        } else if aggregate.function == F::Count {
            (
                lit(1u64),
                Column {
                    name: "count_input".into(),
                    spec: FieldContract::payload(
                        "count_input",
                        T::native(DataType::UInt64),
                        "One per row",
                    ),
                    qualifier: None,
                    physical: None,
                    literal: None,
                },
            )
        } else {
            return Err(internal("aggregate requires an input"));
        };
        if aggregate.null_policy == Null::Reject && column.spec.nullable() {
            let check = LogicalPlanBuilder::from(input.plan.clone())
                .filter(value.clone().is_null())
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            self.checks
                .push((check, "aggregate rejects null input".to_owned()));
        }
        Ok((value, column))
    }

    fn aggregate_order(
        &mut self,
        input: &Planned,
        groups: &[Expr],
        aggregate: &RuleAggregate,
    ) -> Result<Vec<Sort>, RuleError> {
        let mut orders = vec![];
        for (name, ascending) in &aggregate.order_by {
            let column = expr::lookup(input, name)?;
            self.key(column)?;
            orders.push(expr::column_expression(column).sort(*ascending, true));
        }
        if aggregate.function == F::CollectOrdered {
            if orders.is_empty() {
                return Err(internal("ordered collection requires explicit order"));
            }
            let mut tie_keys = groups.to_vec();
            tie_keys.extend(orders.iter().map(|sort| sort.expr.clone()));
            let check = LogicalPlanBuilder::from(input.plan.clone())
                .aggregate(tie_keys, vec![count(lit(1u64)).alias("__pse_order_ties")])
                .map_err(engine)?
                .filter(col("__pse_order_ties").gt(lit(1i64)))
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            self.checks.push((
                check,
                "ordered aggregate keys must determine a unique member order".to_owned(),
            ));
        } else if !orders.is_empty() {
            return Err(internal("only collection accepts ordering"));
        }
        Ok(orders)
    }

    fn aggregate_empty(
        &self,
        aggregate: &RuleAggregate,
        accumulator_type: &DataType,
    ) -> Result<Expr, RuleError> {
        let mut finished = col(aggregate.output_name.as_ref());
        match aggregate.empty_policy {
            // COUNT already returns a nonnull Int64 zero. Coercing it with a
            // UInt64 fallback would unnecessarily introduce Decimal128.
            Empty::Zero if aggregate.function == F::Count => {}
            Empty::Zero if aggregate.function == F::Sum => {
                let zero = ScalarValue::UInt64(Some(0))
                    .cast_to(accumulator_type)
                    .map_err(engine)?;
                finished =
                    datafusion::functions::core::expr_fn::coalesce(vec![finished, lit(zero)]);
            }
            Empty::EmptyList if aggregate.function == F::CollectOrdered => {
                let DataType::List(element) = accumulator_type else {
                    return Err(internal("collection output requires List"));
                };
                let empty = datafusion::arrow::array::ListArray::try_new(
                    element.clone(),
                    datafusion::arrow::buffer::OffsetBuffer::new(vec![0i32, 0].into()),
                    datafusion::arrow::array::new_empty_array(element.data_type()),
                    None,
                )
                .map_err(|error| internal(error.to_string()))?;
                let empty = ScalarValue::List(std::sync::Arc::new(empty));
                // Keep both CASE branches in the aggregate's actual engine
                // storage type. DataFusion 55 scalar casts drop child metadata;
                // the checked final ARRAY cast restores the declared child only
                // after CASE has materialized its result.
                finished =
                    datafusion::functions::core::expr_fn::coalesce(vec![finished, lit(empty)]);
            }
            Empty::Error => {
                finished = self
                    .session
                    .scalar_function("pse_require_nonnull")?
                    .call(vec![finished]);
            }
            _ => {
                return Err(internal(
                    "aggregate empty policy is incompatible with its result",
                ));
            }
        }
        Ok(finished)
    }

    fn finish_aggregation(
        &self,
        grouped_plan: LogicalPlan,
        mut finishing: Vec<Expr>,
        output: &[Column],
        hidden: &[String],
    ) -> Result<LogicalPlan, RuleError> {
        // Cast only after the operator's source contract has established the logical
        // derivation. Result admission still checks every value and nullability.
        for (value, column) in finishing.iter_mut().zip(output) {
            let field = pse_schema::arrow::field_for(self.registry, &column.spec)
                .map_err(|error| internal(error.to_string()))?;
            *value = value
                .clone()
                .cast_to(field.data_type(), grouped_plan.schema())
                .map_err(engine)?
                .alias_with_metadata(
                    column.name.as_ref(),
                    Some(datafusion_common::metadata::FieldMetadata::from(&field)),
                );
        }
        finishing.extend(hidden.iter().map(col));
        let plan = LogicalPlanBuilder::from(grouped_plan)
            .project(finishing)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        Ok(plan)
    }
}

fn native_aggregate(
    input: &Planned,
    aggregate: &RuleAggregate,
    value: &Expr,
    column: &Column,
    orders: Vec<Sort>,
) -> Result<(Expr, T), RuleError> {
    let result = match aggregate.function {
        F::Count => (count(value.clone()), T::native(DataType::UInt64)),
        F::Sum => {
            let ty = match column.spec.data_type() {
                DataType::Int32 | DataType::Int64 => T::native(DataType::Int64),
                DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => {
                    T::native(DataType::UInt64)
                }
                _ => return Err(internal("rule sum requires exact integer input")),
            };
            // The wider engine accumulator prevents intermediate wraparound. The
            // final checked Arrow cast plus nonnull admission rejects overflow.
            let wide = value
                .clone()
                .cast_to(&DataType::Decimal256(76, 0), input.plan.schema())
                .map_err(engine)?;
            (sum(wide), ty)
        }
        F::Min | F::Max => {
            if !column.spec.value_type().admits_exact_key() {
                return Err(internal(
                    "min/max requires an exact ordered type; floating reductions need a numerical policy",
                ));
            }
            (
                if aggregate.function == F::Min {
                    min(value.clone())
                } else {
                    max(value.clone())
                },
                column.spec.value_type().clone(),
            )
        }
        F::CollectOrdered => {
            let mut builder = array_agg(value.clone()).order_by(orders);
            if aggregate.null_policy == Null::SkipMissing {
                builder = builder.filter(value.clone().is_not_null());
            }
            (
                builder.build().map_err(engine)?,
                T::list(column.spec.value_type().clone()),
            )
        }
    };
    Ok(result)
}
