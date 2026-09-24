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
}
/// Bind the built-in registry at the native application boundary. Pure schema
/// consumers use `pse_schema` directly and never construct this SQL state.
/// # Errors
/// Registry construction or native validation-owner installation fails.
pub fn registry() -> Result<&'static pse_schema::Registry, pse_schema::SchemaError> {
    let registry = pse_schema::registry()?;
    bind_defaults(registry).map_err(|error| pse_schema::SchemaError::InvalidDeclaration {
        context: "native validation".into(),
        reason: error.to_string(),
    })?;
    Ok(registry)
}
/// Install the registry's standard native binding once. Custom sessions use `NativeValidation` directly.
/// # Errors
/// The registry owner cannot retain its prepared implementation.
pub fn bind_defaults(registry: &pse_schema::Registry) -> Result<(), pse_relations::RelationError> {
    struct NativeDefault(Arc<dyn ValidationPlanner>);
    let native = registry.derived_implementation(|| {
        Ok(NativeDefault(Arc::new(NativeValidation(
            datafusion::execution::context::SessionContext::new().state(),
        ))))
    })?;
    pse_relations::validate::ValidationContext::install_default(registry, native.0.clone())
}
