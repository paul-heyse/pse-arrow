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
}
