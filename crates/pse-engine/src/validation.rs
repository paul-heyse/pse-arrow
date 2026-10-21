// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual native SQL/function bindings for registry-derived columnar validation.
use datafusion::{
    common::{DFSchema, Result},
    execution::session_state::SessionState,
    logical_expr::{Expr, simplify::SimplifyContext},
    optimizer::simplify_expressions::ExprSimplifier,
    physical_expr::PhysicalExpr,
};
use pse_relations::validate::planner::ValidationPlanner;
use std::sync::Arc;

/// Captured engine state; logical and physical preparation share this exact owner.
#[derive(Clone, Debug)]
pub struct NativeValidation(pub SessionState);
impl ValidationPlanner for NativeValidation {
    fn snapshot(&self) -> Arc<dyn ValidationPlanner> {
        Arc::new(self.clone())
    }
    fn typed_modeling_local_eligible(&self) -> bool {
        // DF55.1.0 create_physical_expr applies analyzer function rewrites even
        // to prebound expressions. They can replace builtin domain semantics.
        // The selected modeling predicates otherwise use primitive operators
        // and prebound get_field/array_length/encode/octet_length kernels. These
        // kernels consume no ConfigOptions. SQL expression/type/query planners
        // are not consulted for this already-bound expression route; the native
        // call uses PhysicalPlanningContext::default. Revisit at the library pin
        // or when the generated specialization admits another field family.
        self.0.analyzer().function_rewrites().is_empty()
    }
    fn create_logical_expr(&self, sql: &str, schema: &DFSchema) -> Result<Expr> {
        self.0.create_logical_expr(sql, schema)
    }
    fn prepare(&self, expression: Expr, schema: &DFSchema) -> Result<Arc<dyn PhysicalExpr>> {
        let expression = expression.resolve_lambda_variables(schema)?.data;
        let context = SimplifyContext::builder()
            .with_schema(Arc::new(schema.clone()))
            .with_config_options(self.0.config_options().clone())
            .with_query_execution_start_time(self.0.execution_props().query_execution_start_time)
            .build();
        let simplifier = ExprSimplifier::new(context);
        self.0.create_physical_expr(
            simplifier.simplify(simplifier.coerce(expression, schema)?)?,
            schema,
        )
    }
    fn prepare_in_scope(
        &self,
        expression: Expr,
        schema: &DFSchema,
        scope: &pse_relations::validate::AdmissionScope,
    ) -> Result<Arc<dyn PhysicalExpr>> {
        let mut state = self.0.clone();
        state.execution_props_mut().query_execution_start_time =
            scope.execution_props().query_execution_start_time;
        Self(state).prepare(expression, schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        arrow::{
            array::{Int64Array, RecordBatch},
            datatypes::{DataType, Field, Schema},
        },
        execution::context::SessionContext,
    };
    use pse_relations::validate::{AdmissionScope, ValidationContext};
    use std::collections::HashMap;

    #[test]
    fn native_validation_uses_selected_admission_clock_without_changing_owner_state() {
        let registry = pse_schema::RegistryBuilder::new().build().unwrap();
        let state = SessionContext::new().state();
        let original_clock = state.execution_props().query_execution_start_time;
        let context = ValidationContext::new(&registry, NativeValidation(state.clone()));
        let schema = Arc::new(Schema::new_with_metadata(
            vec![Field::new("n", DataType::Int64, false)],
            HashMap::from([(
                pse_schema::arrow::KEY_CHECKS.into(),
                r#"{"clock":"to_timestamp_seconds(n) <= now()"}"#.into(),
            )]),
        ));
        let now = context
            .admission_scope()
            .execution_props()
            .query_execution_start_time
            .unwrap();
        let input = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![now.timestamp()]))],
        )
        .unwrap();
        let cancel = pse_columnar::CancellationToken::new();
        let prepared = context.prepare(&registry, schema.clone()).unwrap();
        assert!(prepared.evaluate(&input, 8, &cancel).unwrap().is_valid());
        let mut earlier = context.admission_scope().execution_props().clone();
        earlier.query_execution_start_time = Some(now - std::time::Duration::from_secs(60));
        let earlier = context.for_scope(AdmissionScope::from_execution_props(&earlier));
        let earlier = earlier.prepare(&registry, schema).unwrap();
        assert!(!earlier.evaluate(&input, 8, &cancel).unwrap().is_valid());
        assert!(prepared.evaluate(&input, 8, &cancel).unwrap().is_valid());
        assert_eq!(
            state.execution_props().query_execution_start_time,
            original_clock
        );
        assert_eq!(context.prepared_count().unwrap(), 1);
    }
    #[test]
    fn typed_modeling_eligibility_uses_actual_rewrites_and_prebound_functions() {
        use datafusion::common::{config::ConfigOptions, tree_node::Transformed};
        use datafusion::logical_expr::{
            expr_rewriter::FunctionRewrite, lit, registry::FunctionRegistry,
        };
        #[derive(Debug)]
        struct WrongByteLength;
        impl FunctionRewrite for WrongByteLength {
            fn name(&self) -> &str {
                "wrong_byte_length"
            }
            fn rewrite(
                &self,
                expression: Expr,
                _schema: &DFSchema,
                _config: &ConfigOptions,
            ) -> Result<Transformed<Expr>> {
                Ok(
                    if matches!(&expression, Expr::ScalarFunction(function) if function.func.name() == "octet_length")
                    {
                        Transformed::yes(lit(0_i32))
                    } else {
                        Transformed::no(expression)
                    },
                )
            }
        }
        let registry = pse_schema::RegistryBuilder::new().build().unwrap();
        let mut config = datafusion::execution::context::SessionConfig::new();
        config.options_mut().sql_parser.enable_ident_normalization = false;
        let session = SessionContext::new_with_config(config);
        // A same-named SQL registration does not replace a prebound builtin.
        session.register_udf(datafusion::logical_expr::create_udf(
            "octet_length",
            vec![DataType::Utf8],
            DataType::Int32,
            datafusion::logical_expr::Volatility::Immutable,
            Arc::new(|_| {
                Ok(datafusion::logical_expr::ColumnarValue::Scalar(
                    datafusion::common::ScalarValue::Int32(Some(0)),
                ))
            }),
        ));
        let state = session.state();
        assert!(NativeValidation(state.clone()).typed_modeling_local_eligible());
        let plain = ValidationContext::new(&registry, NativeValidation(state.clone()));
        let field = Field::new("id", DataType::FixedSizeBinary(16), false);
        let schema = Arc::new(Schema::new(vec![field]));
        let input = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(
                datafusion::arrow::array::FixedSizeBinaryArray::try_from_iter(
                    [[0_u8; 16]].into_iter(),
                )
                .unwrap(),
            )],
        )
        .unwrap();
        let cancel = pse_columnar::CancellationToken::new();
        assert!(
            plain
                .prepare(&registry, schema.clone())
                .unwrap()
                .evaluate(&input, 8, &cancel)
                .unwrap()
                .is_valid()
        );
        let mut rewritten = state;
        rewritten
            .register_function_rewrite(Arc::new(WrongByteLength))
            .unwrap();
        assert!(!NativeValidation(rewritten.clone()).typed_modeling_local_eligible());
        let actual = ValidationContext::new(&registry, NativeValidation(rewritten));
        assert!(!actual.typed_modeling_local_eligible());
        assert!(
            !actual
                .prepare(&registry, schema)
                .unwrap()
                .evaluate(&input, 8, &cancel)
                .unwrap()
                .is_valid()
        );
        assert!(NativeValidation(SessionContext::new().state()).typed_modeling_local_eligible());
    }
}
