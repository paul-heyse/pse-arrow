// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit native fixture owner for imported/generated boundary tests.
use datafusion::common::{DFSchema, Result};
use datafusion::execution::session_state::SessionState;
use datafusion::logical_expr::{Expr, simplify::SimplifyContext};
use datafusion::optimizer::simplify_expressions::ExprSimplifier;
use datafusion::physical_expr::PhysicalExpr;
use pse_relations::validate::planner::ValidationPlanner;
use std::sync::Arc;
#[derive(Clone, Debug)]
struct FixturePlanner(SessionState);
impl ValidationPlanner for FixturePlanner {
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
}
pub(crate) fn validation(
    registry: &pse_schema::Registry,
) -> pse_relations::validate::ValidationContext {
    pse_relations::validate::ValidationContext::new(
        registry,
        FixturePlanner(datafusion::prelude::SessionContext::new().state()),
    )
}
