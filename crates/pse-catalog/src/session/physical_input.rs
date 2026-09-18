// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Private logical-input bridge sharing an already planned native child.
use datafusion::{
    arrow::datatypes::{Schema, SchemaRef},
    catalog::{Session, TableProvider},
    common::{DataFusionError, Result},
    logical_expr::{Expr, TableType},
    physical_expr::expressions::Column,
    physical_plan::{ExecutionPlan, projection::ProjectionExec},
};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct PhysicalInput {
    plan: Arc<dyn ExecutionPlan>,
    schema: SchemaRef,
    defaults: std::collections::BTreeMap<String, Expr>,
}
impl PhysicalInput {
    pub(crate) fn native(plan: Arc<dyn ExecutionPlan>) -> Self {
        Self {
            schema: plan.schema(),
            plan,
            defaults: std::collections::BTreeMap::new(),
        }
    }
    pub(crate) fn storage(plan: Arc<dyn ExecutionPlan>) -> Self {
        Self {
            schema: Arc::new(Schema::new(plan.schema().fields().clone())),
            plan,
            defaults: std::collections::BTreeMap::new(),
        }
    }
    pub(crate) fn with_defaults(mut self, source: &dyn TableProvider) -> Self {
        self.defaults = source
            .schema()
            .fields()
            .iter()
            .filter_map(|field| {
                source
                    .get_column_default(field.name())
                    .cloned()
                    .map(|value| (field.name().clone(), value))
            })
            .collect();
        self
    }
}
#[async_trait::async_trait]
impl TableProvider for PhysicalInput {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
    fn get_column_default(&self, name: &str) -> Option<&Expr> {
        self.defaults.get(name)
    }
    fn table_type(&self) -> TableType {
        TableType::Temporary
    }
    async fn scan(
        &self,
        _: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        _: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if !filters.is_empty() {
            return Err(DataFusionError::Plan(
                "physical input does not accept pushed filters".into(),
            ));
        }
        let schema = self.schema();
        let indices = projection
            .cloned()
            .unwrap_or_else(|| (0..schema.fields().len()).collect());
        let expressions: Vec<_> = indices
            .iter()
            .map(|&index| {
                let name = schema.field(index).name();
                let expression: Arc<dyn datafusion::physical_expr::PhysicalExpr> =
                    Arc::new(Column::new(name, index));
                (expression, name.clone())
            })
            .collect();
        let projected = schema.project(&indices)?;
        Ok(Arc::new(ProjectionExec::try_new_with_schema_metadata(
            expressions,
            Arc::clone(&self.plan),
            &projected,
        )?))
    }
}
