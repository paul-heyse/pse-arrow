// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A narrow expression-binding boundary. The engine owns SQL and actual session configuration.
use datafusion_common::{DFSchema, DataFusionError, Result};
use datafusion_expr::{Expr, simplify::SimplifyContext};
use datafusion_optimizer::simplify_expressions::ExprSimplifier;
use datafusion_physical_expr::PhysicalExpr;
use std::sync::Arc;

/// Immutable functions/configuration used to prepare a local declared predicate.
pub trait ValidationPlanner: std::fmt::Debug + Send + Sync {
    /// Retain this exact implementation and its configuration.
    fn snapshot(&self) -> Arc<dyn ValidationPlanner>;
    /// Whether this exact immutable planner preserves the pinned builtin local
    /// semantics used by the generated modeling-row specialization. This is
    /// representation eligibility, never successful validation evidence. Unknown
    /// implementations retain native evaluation; preparing an expression is not
    /// sufficient to opt in. Row SQL and unsupported facets remain ineligible.
    fn typed_modeling_local_eligible(&self) -> bool {
        false
    }
    /// Bind a declared SQL CHECK in an explicitly captured native engine.
    /// # Errors
    /// SQL is unavailable, malformed or not supported by the actual engine.
    fn create_logical_expr(&self, sql: &str, schema: &DFSchema) -> Result<Expr>;
    /// Coerce, simplify and lower one already bound expression.
    /// # Errors
    /// The declared expression cannot execute in this context.
    fn prepare(&self, expression: Expr, schema: &DFSchema) -> Result<Arc<dyn PhysicalExpr>>;
    /// Prepare a scope-sensitive expression using this invocation's timestamp.
    /// Typed-only planners do not silently inherit a stale native timestamp.
    /// # Errors
    /// The planner does not implement invocation-relative SQL preparation.
    fn prepare_in_scope(
        &self,
        _expression: Expr,
        _schema: &DFSchema,
        _scope: &super::AdmissionScope,
    ) -> Result<Arc<dyn PhysicalExpr>> {
        Err(DataFusionError::Plan(
            "scope-sensitive CHECK requires invocation-relative native preparation".into(),
        ))
    }
}
impl ValidationPlanner for Arc<dyn ValidationPlanner> {
    fn snapshot(&self) -> Arc<dyn ValidationPlanner> {
        self.clone()
    }
    fn typed_modeling_local_eligible(&self) -> bool {
        self.as_ref().typed_modeling_local_eligible()
    }
    fn create_logical_expr(&self, sql: &str, schema: &DFSchema) -> Result<Expr> {
        self.as_ref().create_logical_expr(sql, schema)
    }
    fn prepare(&self, expression: Expr, schema: &DFSchema) -> Result<Arc<dyn PhysicalExpr>> {
        self.as_ref().prepare(expression, schema)
    }
    fn prepare_in_scope(
        &self,
        expression: Expr,
        schema: &DFSchema,
        scope: &super::AdmissionScope,
    ) -> Result<Arc<dyn PhysicalExpr>> {
        self.as_ref().prepare_in_scope(expression, schema, scope)
    }
}
/// Built-in typed field predicates need no SQL session. SQL-bearing row declarations
/// require engine installation; an absent binding is an error, never omitted validation.
#[derive(Clone, Debug)]
pub(super) struct LocalPlanner;
impl ValidationPlanner for LocalPlanner {
    fn snapshot(&self) -> Arc<dyn ValidationPlanner> {
        Arc::new(self.clone())
    }
    fn create_logical_expr(&self, _sql: &str, _schema: &DFSchema) -> Result<Expr> {
        Err(DataFusionError::Plan(
            "SQL CHECK requires an engine-bound ValidationContext".into(),
        ))
    }
    fn prepare(&self, expression: Expr, schema: &DFSchema) -> Result<Arc<dyn PhysicalExpr>> {
        let expression = expression.resolve_lambda_variables(schema)?.data;
        let context = SimplifyContext::builder()
            .with_schema(Arc::new(schema.clone()))
            .build();
        let simplifier = ExprSimplifier::new(context);
        let expression = simplifier.simplify(simplifier.coerce(expression, schema)?)?;
        datafusion_physical_expr::create_physical_expr(
            &expression,
            schema,
            &datafusion_expr::execution_props::ExecutionProps::new(),
            &datafusion_expr::physical_planning_context::PhysicalPlanningContext::default(),
        )
    }
}

// Isolated legacy predicate units exercise exact SessionState contracts without
// making SQL a normal dependency of columnar consumers.
#[cfg(test)]
impl ValidationPlanner for datafusion::execution::session_state::SessionState {
    fn snapshot(&self) -> Arc<dyn ValidationPlanner> {
        Arc::new(self.clone())
    }
    fn create_logical_expr(&self, sql: &str, schema: &DFSchema) -> Result<Expr> {
        self.create_logical_expr(sql, schema)
    }
    fn prepare(&self, expression: Expr, schema: &DFSchema) -> Result<Arc<dyn PhysicalExpr>> {
        let expression = expression.resolve_lambda_variables(schema)?.data;
        let context = SimplifyContext::builder()
            .with_schema(Arc::new(schema.clone()))
            .with_config_options(self.config_options().clone())
            .with_query_execution_start_time(self.execution_props().query_execution_start_time)
            .build();
        let simplifier = ExprSimplifier::new(context);
        self.create_physical_expr(
            simplifier.simplify(simplifier.coerce(expression, schema)?)?,
            schema,
        )
    }
    fn prepare_in_scope(
        &self,
        expression: Expr,
        schema: &DFSchema,
        scope: &super::AdmissionScope,
    ) -> Result<Arc<dyn PhysicalExpr>> {
        let mut state = self.clone();
        state.execution_props_mut().query_execution_start_time =
            scope.execution_props().query_execution_start_time;
        ValidationPlanner::prepare(&state, expression, schema)
    }
}
