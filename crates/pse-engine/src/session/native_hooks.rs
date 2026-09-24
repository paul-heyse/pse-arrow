// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pin-specific native hook forwarding. Representation, state, configuration and
//! returned implementations are handled explicitly by the field adapters.

macro_rules! scalar_hooks {
    () => {
        fn aliases(&self) -> &[String] {
            self.native.inner().aliases()
        }
        #[allow(
            deprecated,
            reason = "complete forwarding of the pinned legacy display hook"
        )]
        fn display_name(
            &self,
            args: &[datafusion::logical_expr::Expr],
        ) -> datafusion::common::Result<String> {
            self.native.inner().display_name(args)
        }
        fn schema_name(
            &self,
            args: &[datafusion::logical_expr::Expr],
        ) -> datafusion::common::Result<String> {
            self.native.inner().schema_name(args)
        }
        #[allow(
            deprecated,
            reason = "legacy callers retain native nullability; return_field is authoritative"
        )]
        fn is_nullable(
            &self,
            args: &[datafusion::logical_expr::Expr],
            schema: &dyn datafusion::common::ExprSchema,
        ) -> bool {
            self.native.inner().is_nullable(args, schema)
        }
        fn is_strict(&self) -> bool {
            self.native.inner().is_strict()
        }
        fn simplify(
            &self,
            args: Vec<datafusion::logical_expr::Expr>,
            info: &datafusion::logical_expr::simplify::SimplifyContext,
        ) -> datafusion::common::Result<datafusion::logical_expr::simplify::ExprSimplifyResult> {
            self.native.inner().simplify(args, info)
        }
        fn preimage(
            &self,
            args: &[datafusion::logical_expr::Expr],
            lit_expr: &datafusion::logical_expr::Expr,
            info: &datafusion::logical_expr::simplify::SimplifyContext,
        ) -> datafusion::common::Result<datafusion::logical_expr::preimage::PreimageResult> {
            self.native.inner().preimage(args, lit_expr, info)
        }
        fn short_circuits(&self) -> bool {
            self.native.inner().short_circuits()
        }
        fn conditional_arguments<'a>(
            &self,
            args: &'a [datafusion::logical_expr::Expr],
        ) -> Option<(
            Vec<&'a datafusion::logical_expr::Expr>,
            Vec<&'a datafusion::logical_expr::Expr>,
        )> {
            self.native.inner().conditional_arguments(args)
        }
        fn evaluate_bounds(
            &self,
            input: &[&datafusion::logical_expr::interval_arithmetic::Interval],
        ) -> datafusion::common::Result<datafusion::logical_expr::interval_arithmetic::Interval> {
            self.native.inner().evaluate_bounds(input)
        }
        fn propagate_constraints(
            &self,
            interval: &datafusion::logical_expr::interval_arithmetic::Interval,
            inputs: &[&datafusion::logical_expr::interval_arithmetic::Interval],
        ) -> datafusion::common::Result<
            Option<Vec<datafusion::logical_expr::interval_arithmetic::Interval>>,
        > {
            self.native.inner().propagate_constraints(interval, inputs)
        }
        fn output_ordering(
            &self,
            inputs: &[datafusion::logical_expr::sort_properties::ExprProperties],
        ) -> datafusion::common::Result<datafusion::logical_expr::sort_properties::SortProperties> {
            self.native.inner().output_ordering(inputs)
        }
        fn preserves_lex_ordering(
            &self,
            inputs: &[datafusion::logical_expr::sort_properties::ExprProperties],
        ) -> datafusion::common::Result<bool> {
            self.native.inner().preserves_lex_ordering(inputs)
        }
        fn strictly_order_preserving(
            &self,
            inputs: &[datafusion::logical_expr::sort_properties::ExprProperties],
        ) -> datafusion::common::Result<bool> {
            self.native.inner().strictly_order_preserving(inputs)
        }
        fn coerce_types(
            &self,
            arg_types: &[datafusion::arrow::datatypes::DataType],
        ) -> datafusion::common::Result<Vec<datafusion::arrow::datatypes::DataType>> {
            self.native.inner().coerce_types(arg_types)
        }
        fn struct_field_mapping(
            &self,
            args: &[Option<datafusion::common::ScalarValue>],
        ) -> Option<datafusion::logical_expr::StructFieldMapping> {
            self.native.inner().struct_field_mapping(args)
        }
        fn documentation(&self) -> Option<&datafusion::logical_expr::Documentation> {
            self.native.inner().documentation()
        }
        fn placement(
            &self,
            args: &[datafusion::logical_expr::ExpressionPlacement],
        ) -> datafusion::logical_expr::ExpressionPlacement {
            self.native.inner().placement(args)
        }
    };
}
pub(super) use scalar_hooks;

macro_rules! aggregate_hooks {
    () => {
        fn aliases(&self) -> &[String] {
            self.native.inner().aliases()
        }
        fn schema_name(
            &self,
            params: &datafusion::logical_expr::expr::AggregateFunctionParams,
        ) -> datafusion::common::Result<String> {
            self.native.inner().schema_name(params)
        }
        fn human_display(
            &self,
            params: &datafusion::logical_expr::expr::AggregateFunctionParams,
        ) -> datafusion::common::Result<String> {
            self.native.inner().human_display(params)
        }
        fn window_function_schema_name(
            &self,
            params: &datafusion::logical_expr::expr::WindowFunctionParams,
        ) -> datafusion::common::Result<String> {
            self.native.inner().window_function_schema_name(params)
        }
        fn display_name(
            &self,
            params: &datafusion::logical_expr::expr::AggregateFunctionParams,
        ) -> datafusion::common::Result<String> {
            self.native.inner().display_name(params)
        }
        fn window_function_display_name(
            &self,
            params: &datafusion::logical_expr::expr::WindowFunctionParams,
        ) -> datafusion::common::Result<String> {
            self.native.inner().window_function_display_name(params)
        }
        fn is_nullable(&self) -> bool {
            self.native.inner().is_nullable()
        }
        fn order_sensitivity(&self) -> datafusion::logical_expr::utils::AggregateOrderSensitivity {
            self.native.inner().order_sensitivity()
        }
        fn simplify(
            &self,
        ) -> Option<datafusion::logical_expr::function::AggregateFunctionSimplification> {
            self.native.inner().simplify()
        }
        fn simplify_expr_op_literal(
            &self,
            agg_function: &datafusion::logical_expr::expr::AggregateFunction,
            arg: &datafusion::logical_expr::Expr,
            op: datafusion::logical_expr::Operator,
            lit: &datafusion::logical_expr::Expr,
            arg_is_left: bool,
        ) -> datafusion::common::Result<Option<datafusion::logical_expr::Expr>> {
            self.native
                .inner()
                .simplify_expr_op_literal(agg_function, arg, op, lit, arg_is_left)
        }
        fn coerce_types(
            &self,
            arg_types: &[datafusion::arrow::datatypes::DataType],
        ) -> datafusion::common::Result<Vec<datafusion::arrow::datatypes::DataType>> {
            self.native.inner().coerce_types(arg_types)
        }
        fn is_descending(&self) -> Option<bool> {
            self.native.inner().is_descending()
        }
        fn value_from_stats(
            &self,
            statistics_args: &datafusion::logical_expr::StatisticsArgs<'_>,
        ) -> Option<datafusion::common::ScalarValue> {
            self.native.inner().value_from_stats(statistics_args)
        }
        fn default_value(
            &self,
            data_type: &datafusion::arrow::datatypes::DataType,
        ) -> datafusion::common::Result<datafusion::common::ScalarValue> {
            self.native.inner().default_value(data_type)
        }
        fn supports_null_handling_clause(&self) -> bool {
            self.native.inner().supports_null_handling_clause()
        }
        fn supports_within_group_clause(&self) -> bool {
            self.native.inner().supports_within_group_clause()
        }
        fn documentation(&self) -> Option<&datafusion::logical_expr::Documentation> {
            self.native.inner().documentation()
        }
        fn set_monotonicity(
            &self,
            data_type: &datafusion::arrow::datatypes::DataType,
        ) -> datafusion::logical_expr::SetMonotonicity {
            self.native.inner().set_monotonicity(data_type)
        }
    };
}
pub(super) use aggregate_hooks;

macro_rules! native_identity {
    ($ty:ty) => {
        impl PartialEq for $ty {
            fn eq(&self, other: &Self) -> bool {
                std::sync::Arc::ptr_eq(self.native.inner(), other.native.inner())
                    && std::sync::Arc::ptr_eq(self.origin.inner(), other.origin.inner())
            }
        }
        impl Eq for $ty {}
        impl std::hash::Hash for $ty {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                std::ptr::hash(std::sync::Arc::as_ptr(self.native.inner()), state);
                std::ptr::hash(std::sync::Arc::as_ptr(self.origin.inner()), state);
            }
        }
    };
}
pub(super) use native_identity;
