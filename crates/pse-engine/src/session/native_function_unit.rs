// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pin-specific function/physical-rule units over small Arrow values.
use super::{aggregate, scalar};
use datafusion::{
    arrow::{
        array::{Array, ArrayRef, Int64Array, ListArray, RecordBatch},
        buffer::OffsetBuffer,
        datatypes::{DataType, Field, FieldRef, Schema, SchemaRef},
    },
    common::{Result, ScalarValue, Statistics, stats::Precision, tree_node::TreeNodeRecursion},
    execution::TaskContext,
    logical_expr::{
        AggregateUDF, ColumnarValue, EmitTo, ReturnFieldArgs, ScalarFunctionArgs, StatisticsArgs,
    },
    physical_expr::{
        PhysicalExpr,
        aggregate::{AggregateExprBuilder, AggregateFunctionExpr},
        expressions::Column,
    },
    physical_optimizer::PhysicalOptimizerRule,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, PlanProperties, SendableRecordBatchStream,
        aggregates::{AggregateExec, AggregateMode, PhysicalGroupBy},
        projection::ProjectionExec,
    },
};
use std::{collections::HashMap, sync::Arc};
fn field() -> FieldRef {
    Arc::new(
        Field::new("n", DataType::Int64, true)
            .with_metadata(HashMap::from([("unit".into(), "m".into())])),
    )
}
fn expression(function: Arc<AggregateUDF>) -> AggregateFunctionExpr {
    AggregateExprBuilder::new(function, vec![Arc::new(Column::new("n", 0))])
        .schema(Arc::new(Schema::new(vec![field()])))
        .alias("result")
        .build()
        .unwrap()
}
#[test]
fn min_max_delegate_statistics_order_and_sliding_state() {
    use datafusion::functions_aggregate::min_max::{max_udaf, min_udaf};
    for native in [min_udaf(), max_udaf()] {
        let adapted = aggregate::adapt(&native).unwrap();
        assert_eq!(
            adapted.inner().is_descending(),
            native.inner().is_descending()
        );
        assert_eq!(
            adapted.inner().set_monotonicity(&DataType::Int64),
            native.inner().set_monotonicity(&DataType::Int64)
        );
        assert_eq!(
            adapted.inner().default_value(&DataType::Int64).unwrap(),
            native.inner().default_value(&DataType::Int64).unwrap()
        );
        let schema = Schema::new(vec![field()]);
        let mut stats = Statistics::new_unknown(&schema);
        stats.num_rows = Precision::Exact(3);
        stats.column_statistics[0].min_value = Precision::Exact(ScalarValue::Int64(Some(3)));
        stats.column_statistics[0].max_value = Precision::Exact(ScalarValue::Int64(Some(9)));
        let exprs: Vec<Arc<dyn PhysicalExpr>> = vec![Arc::new(Column::new("n", 0))];
        let args = StatisticsArgs {
            statistics: &stats,
            return_type: &DataType::Int64,
            is_distinct: false,
            exprs: &exprs,
        };
        assert_eq!(
            adapted.inner().value_from_stats(&args),
            native.inner().value_from_stats(&args)
        );
        assert!(adapted.inner().value_from_stats(&args).is_some());
        let adapted_expr = expression(adapted);
        let native_expr = expression(native);
        assert_eq!(adapted_expr.field().metadata(), field().metadata());
        let mut a = adapted_expr.create_sliding_accumulator().unwrap();
        let mut b = native_expr.create_sliding_accumulator().unwrap();
        assert_eq!(a.evaluate().unwrap(), b.evaluate().unwrap());
        let values: ArrayRef = Arc::new(Int64Array::from(vec![Some(7), None, Some(3)]));
        a.update_batch(std::slice::from_ref(&values)).unwrap();
        b.update_batch(std::slice::from_ref(&values)).unwrap();
        assert_eq!(a.evaluate().unwrap(), b.evaluate().unwrap());
        a.retract_batch(std::slice::from_ref(&values)).unwrap();
        b.retract_batch(&[values]).unwrap();
        assert_eq!(a.evaluate().unwrap(), b.evaluate().unwrap());
    }
}
#[test]
fn collection_keeps_fields_through_state_groups_reversal_and_empty() {
    let native = datafusion::functions_aggregate::array_agg::array_agg_udaf();
    let adapted = aggregate::adapt(&native).unwrap();
    let ordered = adapted
        .as_ref()
        .clone()
        .with_beneficial_ordering(true)
        .unwrap()
        .unwrap();
    let datafusion::logical_expr::ReversedUDAF::Reversed(reversed) = ordered.inner().reverse_expr()
    else {
        panic!("reversal required")
    };
    assert!(Arc::ptr_eq(
        aggregate::native(&reversed).unwrap().inner(),
        native.inner()
    ));
    let expr = expression(reversed);
    let mut single = expr.create_accumulator().unwrap();
    let empty = single.evaluate().unwrap();
    assert!(empty.is_null());
    let values: ArrayRef = Arc::new(Int64Array::from(vec![Some(7), None, Some(3)]));
    single.update_batch(std::slice::from_ref(&values)).unwrap();
    let state = single.state().unwrap();
    assert_eq!(
        state[0].data_type(),
        expr.state_fields().unwrap()[0].data_type().clone()
    );
    let mut merged = expr.create_accumulator().unwrap();
    merged
        .merge_batch(
            &state
                .iter()
                .map(|s| s.to_array().unwrap())
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let result = merged.evaluate().unwrap();
    let DataType::List(child) = result.data_type() else {
        panic!("list")
    };
    assert_eq!(child.metadata(), field().metadata());
    let mut groups = expression(aggregate::adapt(&native).unwrap())
        .create_groups_accumulator()
        .unwrap();
    groups.update_batch(&[values], &[0, 0, 1], None, 2).unwrap();
    let state = groups.state(EmitTo::All).unwrap();
    let DataType::List(child) = state[0].data_type() else {
        panic!("list")
    };
    assert_eq!(child.metadata(), field().metadata());
}
fn invoke_element(value: ColumnarValue, input: FieldRef, rows: usize) -> ColumnarValue {
    let native = datafusion::functions_nested::extract::array_element_udf();
    let adapted = scalar::element::function(native);
    let fields = vec![input, Arc::new(Field::new("index", DataType::Int64, false))];
    let output = adapted
        .return_field_from_args(ReturnFieldArgs {
            arg_fields: &fields,
            scalar_arguments: &[None, None],
        })
        .unwrap();
    adapted
        .invoke_with_args(ScalarFunctionArgs {
            args: vec![value, ColumnarValue::Scalar(ScalarValue::Int64(Some(1)))],
            arg_fields: fields,
            number_rows: rows,
            return_field: output,
            config_options: Arc::default(),
        })
        .unwrap()
}
#[test]
fn element_keeps_scalar_fast_path_and_exact_nested_scalar_fields() {
    let inner = Arc::new(
        ListArray::try_new(
            field(),
            OffsetBuffer::new(vec![0, 2].into()),
            Arc::new(Int64Array::from(vec![7, 3])),
            None,
        )
        .unwrap(),
    );
    let inner_field = Arc::new(Field::new("inner", inner.data_type().clone(), true));
    let scalar = invoke_element(
        ColumnarValue::Scalar(ScalarValue::List(inner.clone())),
        inner_field.clone(),
        8192,
    );
    assert!(matches!(
        scalar,
        ColumnarValue::Scalar(ScalarValue::Int64(Some(7)))
    ));
    let array: ArrayRef = inner.clone();
    assert!(matches!(
        invoke_element(ColumnarValue::Array(array), inner_field.clone(), 1),
        ColumnarValue::Array(_)
    ));
    let outer = Arc::new(
        ListArray::try_new(
            inner_field,
            OffsetBuffer::new(vec![0, 1].into()),
            inner,
            None,
        )
        .unwrap(),
    );
    let outer_field = Arc::new(Field::new("outer", outer.data_type().clone(), true));
    let value = invoke_element(
        ColumnarValue::Scalar(ScalarValue::List(outer)),
        outer_field,
        8192,
    );
    let ColumnarValue::Scalar(ScalarValue::List(value)) = value else {
        panic!("nested scalar")
    };
    let DataType::List(child) = value.data_type() else {
        panic!("list")
    };
    assert_eq!(child.metadata(), field().metadata());
}
#[derive(Debug)]
struct ExactStats {
    inner: Arc<dyn ExecutionPlan>,
    stats: Statistics,
}
impl DisplayAs for ExactStats {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ExactStatsFixture")
    }
}
impl ExecutionPlan for ExactStats {
    fn name(&self) -> &'static str {
        "ExactStatsFixture"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        self.inner.properties()
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![]
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        assert!(children.is_empty());
        Ok(self)
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(&Arc<dyn PhysicalExpr>) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        Ok(TreeNodeRecursion::Continue)
    }
    fn statistics_from_inputs(
        &self,
        _: &[Arc<Statistics>],
        _: &datafusion::physical_plan::StatisticsArgs,
    ) -> Result<Arc<Statistics>> {
        Ok(Arc::new(self.stats.clone()))
    }
    fn execute(&self, _: usize, _: Arc<TaskContext>) -> Result<SendableRecordBatchStream> {
        panic!("statistics substitution must remove source execution")
    }
}
#[tokio::test]
async fn native_statistics_substitution_retains_selected_field_metadata() {
    let schema: SchemaRef = Arc::new(Schema::new(vec![field()]));
    let batch =
        RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(vec![3, 7]))]).unwrap();
    let inner = datafusion::datasource::memory::MemorySourceConfig::try_new_exec(
        &[vec![batch]],
        schema.clone(),
        None,
    )
    .unwrap();
    let mut stats = Statistics::new_unknown(&schema);
    stats.num_rows = Precision::Exact(2);
    stats.column_statistics[0].min_value = Precision::Exact(ScalarValue::Int64(Some(3)));
    let input = Arc::new(ExactStats { inner, stats });
    let expr = Arc::new(expression(
        aggregate::adapt(&datafusion::functions_aggregate::min_max::min_udaf()).unwrap(),
    ));
    let plan: Arc<dyn ExecutionPlan> = Arc::new(
        AggregateExec::try_new(
            AggregateMode::Single,
            PhysicalGroupBy::new_single(vec![]),
            vec![expr],
            vec![None],
            input,
            schema,
        )
        .unwrap(),
    );
    let rule = super::physical_fields::AggregateFields {
        native: Arc::new(
            datafusion::physical_optimizer::aggregate_statistics::AggregateStatistics::new(),
        ),
    };
    let optimized = rule
        .optimize(plan, &datafusion::common::config::ConfigOptions::default())
        .unwrap();
    assert!(optimized.is::<ProjectionExec>());
    assert_eq!(optimized.schema().field(0).metadata(), field().metadata());
    let result = datafusion::physical_plan::collect(optimized, Arc::new(TaskContext::default()))
        .await
        .unwrap();
    assert_eq!(
        result[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        3
    );
}

#[test]
fn named_struct_delegates_argument_mapping_and_keeps_child_meaning() {
    let native = datafusion::functions::core::named_struct();
    let adapted = scalar::structure::function(native.clone());
    let name = ScalarValue::Utf8(Some("selected".into()));
    let values = [Some(name.clone()), None];
    let native_mapping = native.struct_field_mapping(&values).unwrap();
    let adapted_mapping = adapted.struct_field_mapping(&values).unwrap();
    assert_eq!(native_mapping.fields, adapted_mapping.fields);
    // The native hook constructs a fresh stateless accessor on each call.
    assert_eq!(
        native_mapping.field_accessor,
        adapted_mapping.field_accessor
    );
    let args = vec![
        Arc::new(Field::new("label", DataType::Utf8, false)),
        field(),
    ];
    let output = adapted
        .return_field_from_args(ReturnFieldArgs {
            arg_fields: &args,
            scalar_arguments: &[Some(&name), None],
        })
        .unwrap();
    let DataType::Struct(fields) = output.data_type() else {
        panic!("struct")
    };
    assert_eq!(fields[0].metadata(), field().metadata());
    assert!(!output.is_nullable());
    let value = adapted
        .invoke_with_args(ScalarFunctionArgs {
            args: vec![
                ColumnarValue::Scalar(name),
                ColumnarValue::Scalar(ScalarValue::Int64(Some(3))),
            ],
            arg_fields: args,
            number_rows: 1,
            return_field: output.clone(),
            config_options: Arc::default(),
        })
        .unwrap();
    assert_eq!(&value.data_type(), output.data_type());
}

#[test]
fn ordered_collection_reversal_and_merge_follow_the_returned_native_implementation() {
    let native = datafusion::functions_aggregate::array_agg::array_agg_udaf();
    let adapted = aggregate::adapt(&native).unwrap();
    let mut results = Vec::new();
    for function in [native, adapted] {
        let column: Arc<dyn PhysicalExpr> = Arc::new(Column::new("n", 0));
        let expression = AggregateExprBuilder::new(function, vec![column.clone()])
            .schema(Arc::new(Schema::new(vec![field()])))
            .alias("ordered")
            .order_by(vec![datafusion::physical_expr::PhysicalSortExpr::new(
                column,
                datafusion::arrow::compute::SortOptions::default(),
            )])
            .build()
            .unwrap();
        let reversed = Arc::new(expression)
            .with_beneficial_ordering(true)
            .unwrap()
            .unwrap()
            .reverse_expr()
            .unwrap();
        let mut partial = reversed.create_accumulator().unwrap();
        let values: ArrayRef = Arc::new(Int64Array::from(vec![7, 3, 9]));
        partial.update_batch(&[values.clone(), values]).unwrap();
        let state = partial.state().unwrap();
        let mut final_accumulator = reversed.create_accumulator().unwrap();
        final_accumulator
            .merge_batch(
                &state
                    .iter()
                    .map(|s| s.to_array().unwrap())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let ScalarValue::List(result) = final_accumulator.evaluate().unwrap() else {
            panic!("list")
        };
        results.push(result);
    }
    assert_eq!(results[0].values(), results[1].values());
    let DataType::List(child) = results[1].data_type() else {
        panic!("list")
    };
    assert_eq!(child.metadata(), field().metadata());
}
